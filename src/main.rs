use console::style;

pub mod app;
pub mod common;
pub mod utils;
mod wizard;

use wizard::Wizard;

fn main() {
	println!("Fossfox v{} 🦊", env!("CARGO_PKG_VERSION"));
	println!("{}\n", style(env!("CARGO_PKG_HOMEPAGE")).blue().bold());

	match Wizard::new() {
		Ok(mut wizard) => {
			if let Err(e) = wizard.start() {
				println!("Error: {e}");
			}
		}
		Err(e) => println!("Error: {e}"),
	}
}
