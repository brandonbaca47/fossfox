use chrono::Datelike;
use console::Style;
use dialoguer::{theme::ColorfulTheme, Confirm, FuzzySelect, Input, Select};
use email_address::EmailAddress;
use eyre::Result;
use std::{collections::HashSet, str::FromStr, time::SystemTime};
use url::Url;

use crate::{
	app::App,
	common::{Company, Item, Product},
	utils,
};

pub struct Wizard {
	app: App,
	theme: ColorfulTheme,
	data_changed: bool,
}

impl Wizard {
	pub fn new() -> Result<Self> {
		Ok(Wizard {
			app: App::new().unwrap(),
			theme: ColorfulTheme {
				values_style: Style::new().green().bright(),
				..ColorfulTheme::default()
			},
			data_changed: false,
		})
	}

	pub fn start(&mut self) -> Result<()> {
		let mut slug = "".to_string();
		let mut domain = "".to_string();
		let mut url = "".to_string();

		Input::with_theme(&self.theme)
			.with_prompt("Company website (eg: example.com)")
			.validate_with(|input: &String| -> Result<(), &str> {
				match utils::parse_url(input) {
					Ok(Some((s, d, u))) if !s.is_empty() && !d.is_empty() && !u.is_empty() => {
						slug = s;
						domain = d;
						url = u;

						Ok(())
					}
					_ => Err("Invalid website"),
				}
			})
			.interact()?;

		if let Some(company) = self.app.companies.get(&slug) {
			self.app.company = Some(company.clone());
		} else {
			println!(
				"\n👋 Cool, looks like we don't list your company yet. Let's add it real quick:"
			);
			self.app.company = Some(self.company(&slug, &domain, &url)?);
			self.data_changed = true;
		}

		self.menu()?;

		if self.data_changed {
			if let Some(company) = self.app.company.clone() {
				self.app.write_company(&company)?;
			}

			self.app.show_instructions();
		}

		Ok(())
	}

	pub fn menu(&self) -> Result<()> {
		if let Some(company) = self.app.company.clone() {
			let action = Select::with_theme(&self.theme)
				.with_prompt(format!("What would you like to do with {}?", company.name))
				.items(&["Add/remove jobs", "Edit company metadata", "Quit"])
				.default(0)
				.interact()?;

			match action {
				0 => self.menu_jobs()?,
				1 => println!("@todo edit company"),
				_ => {}
			}
		}

		Ok(())
	}

	pub fn menu_jobs(&self) -> Result<()> {
		if let Some(company) = self.app.company.clone() {
			let no_jobs = company.jobs.is_empty();
			let items =
				if no_jobs { vec!["Add", "Quit"] } else { vec!["Add", "Edit", "Remove", "Quit"] };

			let action = Select::with_theme(&self.theme)
				.with_prompt(format!(
					"{} has {} job{} listed",
					company.name,
					company.jobs.len(),
					if company.jobs.len() == 1 { "" } else { "s" }
				))
				.items(&items)
				.default(0)
				.interact()?;

			match action {
				0 => println!("@todo add jobs"),
				1 if !no_jobs => println!("@todo edit jobs"),
				2 => println!("@todo remove jobs"),
				_ => {}
			}
		}

		Ok(())
	}
	pub fn company(&self, slug: &str, domain: &str, url: &str) -> Result<Company> {
		let name: String =
			Input::with_theme(&self.theme).with_prompt("Company name (eg: Example)").interact()?;

		let mut at = "".to_string();
		Input::with_theme(&self.theme)
			.with_prompt("Email address for job applications")
			.default(format!("careers@{domain}"))
			.validate_with(|input: &String| -> Result<(), &str> {
				if let Ok(email) = EmailAddress::from_str(input) {
					at = email.local_part().to_string();
					Ok(())
				} else {
					Err("invalid email")
				}
			})
			.interact()?;

		let building: String = Input::with_theme(&self.theme)
			.with_prompt("What are you building in 5 words or less")
			.validate_with(|input: &String| -> Result<(), &str> {
				let words =
					input.split_whitespace().map(|s| s.to_string()).collect::<Vec<String>>();
				if words.is_empty() || words.len() > 5 {
					Err("invalid length")
				} else {
					Ok(())
				}
			})
			.interact()?;

		let services = vec!["Github", "Gitlab"];
		let service = Select::with_theme(&self.theme)
			.with_prompt("Does your org use Github or Gitlab?")
			.items(&services)
			.default(0)
			.interact()?;

		let service_username: String = Input::with_theme(&self.theme)
			.with_prompt(format!("{} username", services[service]))
			.interact()?;

		println!("\n👔 Some useful info for applicants (all optional):");

		let twitter_username: String =
			Input::with_theme(&self.theme).with_prompt("Twitter username").interact()?;

		let founded = Input::with_theme(&self.theme)
			.with_prompt("What year was it founded?")
			.validate_with(|input: &String| -> Result<(), &str> {
				if let Ok(year) = input.parse::<u16>() {
					if year >= 1900 && i32::from(year) <= chrono::Utc::now().year() {
						return Ok(());
					}
				}

				Err("Invalid year")
			})
			.allow_empty(true)
			.default(chrono::Utc::now().year().to_string())
			.interact()?
			.parse::<u16>()
			.unwrap();

		let headcount = Input::with_theme(&self.theme)
			.with_prompt("What's the company's approximate headcount?")
			.validate_with(|input: &String| -> Result<(), &str> {
				if input.parse::<u16>().is_ok() {
					Ok(())
				} else {
					Err("Invalid headcount")
				}
			})
			.allow_empty(true)
			.default("0".to_string())
			.interact()?
			.parse::<u16>()
			.unwrap();

		let offices = {
			let mut office_ids = HashSet::new();

			let done = "Done".to_string();

			let locations = self.app.locations.values().cloned().collect::<Vec<Item>>();
			let mut all_offices =
				locations.iter().map(|loc| loc.name.clone()).collect::<Vec<String>>();

			all_offices.insert(0, done);

			loop {
				let index = FuzzySelect::with_theme(&self.theme)
					.with_prompt("Any physical offices (besides remote)?")
					.default(0)
					.items(&all_offices[..])
					.max_length(10)
					.interact()
					.unwrap();

				if index == 0 {
					break;
				}

				if let Some(office) = all_offices.get(index) {
					if let Some(location) = locations.iter().find(|loc| loc.name == *office) {
						office_ids.insert(location.id.clone());
					}
				}
			}

			office_ids
		};

		let products = {
			let mut ret = HashSet::new();

			println!("\n✨ We can showcase product(s) that you're working on so applicants can check them out.");

			if Confirm::with_theme(&self.theme)
				.with_prompt("Would you like to list them now?")
				.interact()?
			{
				let count = Select::with_theme(&self.theme)
					.with_prompt("How many products are you working on?")
					.items(&["1", "2", "3+"])
					.default(0)
					.interact()? + 1;

				for i in 1..=count {
					ret.insert(Product {
						name: Input::with_theme(&self.theme)
							.with_prompt(format!("Product #{i} Name"))
							.interact()?,
						url: Input::with_theme(&self.theme)
							.with_prompt(format!("Product #{i} URL"))
							.validate_with(|input: &String| -> Result<(), &str> {
								if Url::parse(input).is_ok() {
									Ok(())
								} else {
									Err("invalid url")
								}
							})
							.interact()?,
					});
				}
			}

			ret
		};

		let mut socials = HashSet::new();
		if service == 0 {
			socials.insert(format!("https://github.com/{service_username}"));
		} else {
			socials.insert(format!("https://gitlab.com/{service_username}"));
		}
		if !twitter_username.is_empty() {
			socials.insert(format!("https://twitter.com/{twitter_username}"));
		}

		let jobs = vec![];

		Ok(Company {
			slug: slug.to_string(),
			name,
			url: url.to_string(),
			at,
			building,
			products,
			socials,
			offices,
			headcount,
			founded,
			jobs,
			updated: SystemTime::now().into(),
		})
	}
}
