// SPDX-License-Identifier: WTFPL

use std::{
	collections::HashMap,
	sync::{Arc, Mutex},
	time::Duration,
};

use imageproc::{
	drawing::draw_text,
	image::{ImageFormat, Rgb, RgbImage},
};
use tiny_http::{Response, Server};

fn main() {
	let db_path = if false { "fuck.json" } else { "/data/fuck.json" };

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
			let hit_count = backup_hit_count.lock().unwrap().clone();
			let v = serde_json::to_string(&hit_count).unwrap();
			let _ = std::fs::write(db_path, &v);
			// println!("{}", v);
		}
	});

	let server = Server::http("0.0.0.0:8080").unwrap();
	// println!("listening...");

	for request in server.incoming_requests() {
        let url = request.url().split("?").next().unwrap();
		if url == "/" {
			let _ = request.respond(Response::from_string(""));
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
		if url.len() > 20 {
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
		if let Ok(_) = new_image.write_to(&mut cursor, ImageFormat::Jpeg) {
			let _ = request.respond(Response::from_data(cursor.into_inner()));
		} else {
			let _ = request.respond(Response::from_string("failed"));
		}
	}
}
