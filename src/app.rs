use chrono::{DateTime, Datelike, Utc};
use console::Style;
use dialoguer::{theme::ColorfulTheme, FuzzySelect, Input, Select};
use email_address::*;
use glob::glob;
use std::{
	collections::{HashMap, HashSet},
	env,
	error::Error,
	fs,
	str::FromStr,
	time::SystemTime,
};

use crate::{
	common::{AppError, Company, Item},
	utils,
};

pub struct App {
	pub locations: HashMap<String, Item>,
	pub positions: HashMap<String, Item>,
	pub tech: HashMap<String, Item>,
	pub companies: HashMap<String, Company>,
	company: Option<Company>,
}

impl App {
	pub fn new() -> Result<Self, AppError> {
		let mut app = App {
			locations: HashMap::new(),
			positions: HashMap::new(),
			tech: HashMap::new(),
			companies: HashMap::new(),
			company: None,
		};

		app.locations = app.read_items("locations")?;
		app.positions = app.read_items("positions")?;
		app.tech = app.read_items("tech")?;

		app.read_companies()?;

		Ok(app)
	}

	fn read_items(&self, filename: &str) -> Result<HashMap<String, Item>, AppError> {
		let mut file_path = env::current_dir().unwrap();
		file_path.push("data");
		file_path.push(format!("{filename}.json"));

		let file_contents: String = fs::read_to_string(file_path).unwrap().parse().unwrap();
		let data: Vec<Item> = serde_json::from_str(&file_contents).unwrap();

		Ok(data.into_iter().map(|i| (i.id.clone(), i)).collect::<HashMap<String, Item>>())
	}

	fn read_companies(&mut self) -> Result<(), AppError> {
		let mut companies = HashMap::new();

		for entry in glob("data/companies/**/*.json").unwrap() {
			match entry {
				Ok(path) => {
					let file_contents: String =
						fs::read_to_string(path.clone()).unwrap().parse().unwrap();
					let mut company: Company = serde_json::from_str(&file_contents).unwrap();
					company.slug = path.file_stem().unwrap().to_str().unwrap().to_string();
					companies.insert(company.slug.clone(), company);
				}
				Err(e) => println!("{:?}", e),
			}
		}

		self.companies = companies;
		Ok(())
	}

	pub fn run(&mut self) -> Result<Option<Company>, Box<dyn Error>> {
		println!("Fossfox v{} 🦊", env!("CARGO_PKG_VERSION"));

		let theme = ColorfulTheme {
			values_style: Style::new().green().bright(),
			..ColorfulTheme::default()
		};

		let mut slug = "".to_string();
		let mut domain = "".to_string();
		let mut url = "".to_string();
		Input::with_theme(&theme)
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

		if let Some(company) = self.companies.get(&slug) {
			self.company = Some(company.clone());
		} else {
			println!(
				"\n👋 Cool, looks like we don't list your company yet. Let's add it real quick:"
			);

			let name: String =
				Input::with_theme(&theme).with_prompt("Company name (eg: Example)").interact()?;

			let mut at = "".to_string();
			Input::with_theme(&theme)
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

			let building: String = Input::with_theme(&theme)
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
			let service = Select::with_theme(&theme)
				.with_prompt("Does your org use Github or Gitlab?")
				.items(&services)
				.default(0)
				.interact()?;

			let service_username: String = Input::with_theme(&theme)
				.with_prompt(format!("{} username", services[service]))
				.interact()?;

			println!("\n👔 Some useful info for applicants (all optional):");

			let twitter_username: String =
				Input::with_theme(&theme).with_prompt("Twitter username").interact()?;

			let founded = Input::with_theme(&theme)
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

			let headcount = Input::with_theme(&theme)
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
				let mut offices = HashSet::new();

				let done = "Done".to_string();

				let locations = self.locations.values().cloned().collect::<Vec<Item>>();
				let mut all_offices =
					locations.iter().map(|loc| loc.name.clone()).collect::<Vec<String>>();

				all_offices.insert(0, done);

				loop {
					let index = FuzzySelect::with_theme(&theme)
						.with_prompt("Any physical offices (besides remote)?")
						.default(0)
						.items(&all_offices[..])
						.interact()
						.unwrap();

					if index == 0 {
						break;
					}

					if let Some(office) = all_offices.get(index) {
						offices.insert(office.clone());
						if let Some(location) = locations.iter().find(|loc| loc.name == *office) {
							office_ids.insert(location.id.clone());
						}
					}
				}

				if !office_ids.is_empty() {
					println!(
						"Offices in:\n{}",
						offices
							.into_iter()
							.map(|o| format!("- {o}"))
							.collect::<Vec<String>>()
							.join("\n")
					);
				}

				office_ids
			};

			let products = HashSet::new();

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

			let now = SystemTime::now();
			let now: DateTime<Utc> = now.into();

			let updated = now;

			self.company = Some(Company {
				slug,
				name,
				url,
				at,
				building,
				products,
				socials,
				offices,
				headcount,
				founded,
				jobs,
				updated,
			});
		}

		self.show_menu()
	}

	fn show_menu(&mut self) -> Result<Option<Company>, Box<dyn Error>> {
		println!("showing menu");
		Ok(self.company.clone())
	}
}
