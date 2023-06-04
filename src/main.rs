pub mod app;
pub mod common;
pub mod utils;
mod wizard;

use wizard::Wizard;

fn main() {
	println!("Fossfox v{} 🦊", env!("CARGO_PKG_VERSION"));
	match Wizard::new() {
		Ok(mut wizard) => {
			if let Err(e) = wizard.start() {
				println!("Error: {e}");
			}
		}
		Err(e) => println!("Error: {e}"),
	}
}
