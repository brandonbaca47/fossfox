use chrono::{Duration, Utc};
use clap::Parser;
use console::style;
use std::time::SystemTime;

pub mod app;
pub mod common;
pub mod utils;
mod wizard;

use wizard::Wizard;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
	/// List all expired jobs
	#[arg(long, default_value_t = false)]
	outdated: bool,

	/// Bump all expired jobs
	#[arg(long, default_value_t = false)]
	bump: bool,
}

fn main() {
	println!("Fossfox v{} 🦊", env!("CARGO_PKG_VERSION"));
	println!("{}\n", style(env!("CARGO_PKG_HOMEPAGE")).blue().bold());

	let args = Args::parse();

	match Wizard::new() {
		Ok(mut wizard) => match args {
			Args { outdated: true, .. } => {
				let mut update_slugs = vec![];

				for (domain, company) in &wizard.app.companies {
					if company.updated + Duration::days(30) < Utc::now() {
						println!("{}:", company.name);
						for job in company.jobs.iter() {
							if let Some(position) = wizard.app.positions.get(&job.position) {
								println!("- {} @ `{}`", position.name, job.url);
							}
						}

						if args.bump {
							update_slugs.push(domain.clone());
						}
					}
				}

				for slug in update_slugs.into_iter() {
					if let Some(company) = wizard.app.companies.get(&slug) {
						let mut c = company.clone();
						c.updated = SystemTime::now().into();
						wizard.app.write_company(&c).unwrap();
					}
				}
			}
			_ => {
				if let Err(e) = wizard.start() {
					println!("Error: {e}");
				}
			}
		},
		Err(e) => println!("Error: {e}"),
	}
}
