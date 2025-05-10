// SPDX-License-Identifier: WTFPL

use std::{
	collections::HashMap,
	io::Seek,
	str::FromStr,
	sync::{Arc, Mutex},
	time::Duration,
};

use imageproc::{
	drawing::draw_text,
	image::{ImageFormat, Rgb, RgbImage},
};
use rand::Rng;
use tiny_http::{Response, Server, StatusCode};

fn main() {
	let db_path = if std::env::var_os("FLY_APP_NAME").is_none() {
		"fuck.json"
	} else {
		"/data/fuck.json"
	};

	let dump_secret = std::env::var("DUMP_SECRET").unwrap_or_else(|_| "test".into());

	let mut image = RgbImage::new(92, 14);
	image.fill(0); // set to black
	let font = ab_glyph::FontRef::try_from_slice(include_bytes!("VCR_OSD_MONO_1.001.ttf")).unwrap();

	let hit_count: Arc<Mutex<HashMap<String, u32>>> = Arc::new(Mutex::new(HashMap::new()));
	if let Ok(s) = std::fs::read_to_string(db_path) {
		*hit_count.lock().unwrap() = serde_json::from_str(&s).unwrap();
	}

	let backup_hit_count = hit_count.clone();
	std::thread::spawn(move || {
		loop {
			std::thread::sleep(Duration::from_secs(60));
			let hit_count = { backup_hit_count.lock().unwrap().clone() };
			let v = serde_json::to_string(&hit_count).unwrap();
			let _ = std::fs::write(db_path, &v);
			// println!("{}", v);
		}
	});

	let server = Server::http("0.0.0.0:8080").unwrap();
	// println!("listening...");
	let jpg_content_type = vec![tiny_http::Header::from_str("Content-Type: image/jpg").unwrap()];

	// look into cloudflare's hotlink protection
	let blocked = Response::new(StatusCode(403), vec![], std::io::Cursor::new(""), None, None);

	for request in server.incoming_requests() {
		let url = request.url().split("?").next().unwrap();
		if url == "/" {
			let _ = request.respond(Response::from_string("https://github.com/srcwr/hc"));
			continue;
		}
		if request.url().starts_with("/dump.json?key=") {
			if let Some(secret) = request.url().split('=').nth(1) {
				// random sleep to prevent timing attacks on the secret 🕵️‍♀️
				std::thread::sleep(Duration::from_micros(rand::thread_rng().gen_range(500..1999)));
				if secret == dump_secret {
					let hit_count = { hit_count.lock().unwrap().clone() };
					let _ = request.respond(Response::from_string(serde_json::to_string(&hit_count).unwrap()));
				} else {
					let _ = request.respond(Response::from_string(""));
				}
			}
			continue;
		}
		if !url.starts_with("/hc/") || !url.ends_with(".jpg") {
			let _ = request.respond(Response::from_string(""));
			continue;
		}
		// "/hc/home.jpg"
		// "/hc/maps.jpg"
		// "/hc/hashed.jpg"
		// "/hc/69.jpg"
		// "/hc/ksf.jpg"
		// "/hc/czar.jpg"
		// "/hc/maps_ksfthings.jpg"
		if url.len() > 30 {
			let _ = request.respond(Response::from_string("too big"));
			continue;
		}

		let thing = url[4..url.len() - 4].to_ascii_lowercase();
		if thing.is_empty() {
			continue;
		}
		// println!("{}", thing);

		let count = {
			let mut hit_count = hit_count.lock().unwrap();
			let entry = hit_count.entry(thing).or_default();
			*entry += 1;
			*entry
		};

		let text = format!(
			"{:03},{:03},{:03}",
			count / 1_000_000,
			(count / 1000) % 1000,
			count % 1000
		);
		let new_image = draw_text(&image, Rgb([255, 255, 255]), 0, 0, 14.0, &font, &text);

		let mut cursor = std::io::Cursor::new(vec![]);
		if new_image.write_to(&mut cursor, ImageFormat::Jpeg).is_ok() {
			let _ = cursor.seek(std::io::SeekFrom::Start(0)); // necessary?
			let resp = Response::new(StatusCode(200), jpg_content_type.clone(), cursor, None, None);
			let _ = request.respond(resp);
		} else {
			let _ = request.respond(Response::from_string("failed"));
		}
	}
}
