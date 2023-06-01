use std::error::Error;

use console::Style;
use dialoguer::{theme::ColorfulTheme, Input};

#[derive(Debug)]
#[allow(dead_code)]
struct Config {
	website: String,
}

fn init_config() -> Result<Option<Config>, Box<dyn Error>> {
	let theme =
		ColorfulTheme { values_style: Style::new().yellow().dim(), ..ColorfulTheme::default() };
	println!("Fossfox CLI (work-in-progress)");

	let website = Input::with_theme(&theme).with_prompt("What's the website?").interact()?;

	Ok(Some(Config { website }))
}

fn main() {
	match init_config() {
		Ok(None) => println!("Aborted."),
		Ok(Some(config)) => println!("{:#?}", config),
		Err(err) => println!("error: {}", err),
	}
}
