use console::Style;
use dialoguer::{theme::ColorfulTheme, Input};
use serde::{Deserialize, Serialize};
use slugify::slugify;
use std::{env, error::Error, fs};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Config {
	#[serde(skip_serializing)]
	slug: String,
	name: String,
	fqdn: String,
}

#[derive(Error, Debug)]
pub enum AppError {
	#[error("Could not write JSON file to {0}")]
	CouldNotWrite(String),
	#[error("Unknown")]
	Unknown,
}

fn init_config() -> Result<Option<Config>, Box<dyn Error>> {
	let theme =
		ColorfulTheme { values_style: Style::new().green().bright(), ..ColorfulTheme::default() };
	println!("Fossfox CLI (work-in-progress; do not use yet)");

	let fqdn =
		Input::with_theme(&theme).with_prompt("Company website (eg: example.com)").interact()?;

	let name: String =
		Input::with_theme(&theme).with_prompt("Company name (eg: Example)").interact()?;
	let slug = slugify!(&name);

	Ok(Some(Config { slug, name, fqdn }))
}

fn write_file(config: &Config) -> Result<(), AppError> {
	let file_contents = serde_json::to_string_pretty(config).unwrap();

	let mut file_path = env::current_dir().unwrap();
	file_path.push("data");
	file_path.push("companies");
	file_path.push(config.slug.chars().next().unwrap().to_string());
	file_path.push(format!("{}.json", config.slug));

	if let Some(p) = file_path.parent() {
		fs::create_dir_all(p).unwrap();
	}
	fs::write(file_path, file_contents).unwrap();

	Ok(())
}

fn print_success() -> Result<(), AppError> {
	println!("success");
	Ok(())
}

fn main() {
	match init_config() {
		Ok(None) => println!("Aborted."),
		Ok(Some(config)) => {
			write_file(&config).unwrap();
			print_success().unwrap();
		}
		Err(err) => println!("Error: {}", err),
	}
}
