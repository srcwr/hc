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
use tiny_http::{Response, Server, StatusCode};

fn main() {
	let db_path = if std::env::var_os("FLY_APP_NAME").is_none() {
		"fuck.json"
	} else {
		"/data/fuck.json"
	};

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
			std::thread::sleep(Duration::from_secs(300));
			let hit_count = { backup_hit_count.lock().unwrap().clone() };
			let v = serde_json::to_string(&hit_count).unwrap();
			let _ = std::fs::write(db_path, &v);
			// println!("{}", v);
		}
	});

	let server = Server::http("0.0.0.0:8080").unwrap();
	// println!("listening...");
	let jpg_content_type = vec![tiny_http::Header::from_str("Content-Type: image/jpg").unwrap()];

	for request in server.incoming_requests() {
		if request.url().len() > 100 {
			let _ = request.respond(Response::new(
				StatusCode(403),
				vec![],
				std::io::Cursor::new("url too long"),
				None,
				None,
			));
			continue;
		}

		if request.url() == "/" {
			let _ = request.respond(Response::from_string("https://github.com/srcwr/hc"));
			continue;
		}
		if request.url() == "/dump.json" {
			let hit_count = { hit_count.lock().unwrap().clone() };
			let _ = request.respond(Response::from_string(serde_json::to_string(&hit_count).unwrap()));
			continue;
		}

		if let Some(referer) = request.headers().iter().find(|h| h.field.equiv("referer"))
			&& let Ok(referer) = referer.value.as_str().parse::<url::Url>()
			&& let Some(host) = referer.domain()
			&& host.to_lowercase().ends_with("fastdl.me")
		{
			// yay!
		} else {
			let _ = request.respond(Response::new(
				StatusCode(403),
				vec![],
				std::io::Cursor::new("invalid referer"),
				None,
				None,
			));
			continue;
		}

		let Some(thing) = request.url().strip_prefix("/hc/") else {
			let _ = request.respond(Response::from_string("invalid url"));
			continue;
		};
		let Some(thing) = thing.strip_suffix(".jpg") else {
			let _ = request.respond(Response::from_string("invalid url"));
			continue;
		};

		let count = {
			let mut hit_count = hit_count.lock().unwrap();
			if let Some(v) = hit_count.get_mut(thing) {
				*v += 1;
				*v
			} else {
				let _ = request.respond(Response::from_string(""));
				continue;
			}
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
			let _ = request.respond(Response::from_string("failed to generate image"));
		}
	}
}
