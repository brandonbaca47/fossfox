use std::{env, fs};

mod app;
pub mod common;
pub mod utils;

use app::App;

fn main() {
	let mut app = App::new().unwrap();
	match app.run() {
		Err(err) => println!("Error: {}", err),
		Ok(None) => println!("Aborted."),
		Ok(Some(company)) => {
			// write file
			let file_contents = serde_json::to_string_pretty(&company).unwrap();
			let mut file_path = env::current_dir().unwrap();
			file_path.push("data");
			file_path.push("companies");
			file_path.push(company.slug.chars().next().unwrap().to_string());
			file_path.push(format!("{}.json", company.slug));
			if let Some(p) = file_path.parent() {
				fs::create_dir_all(p).unwrap();
			}
			fs::write(file_path, file_contents).unwrap();

			// message
			println!("success");
		}
	}
}
