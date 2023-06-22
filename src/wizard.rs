use chrono::Datelike;
use console::Style;
use dialoguer::{theme::ColorfulTheme, Confirm, FuzzySelect, Input, Select};
use email_address::EmailAddress;
use eyre::Result;
use std::{
	borrow::BorrowMut,
	collections::{HashMap, HashSet},
	str::FromStr,
	time::SystemTime,
};
use url::Url;

use crate::{
	app::App,
	common::{Company, Currency, Item, Job, Level, Product, Range, Salary, Type},
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
			.with_prompt("Company domain (eg: example.com)")
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
			self.app.company = Some(self.company(&slug, Some(url))?);
			println!("\n💾 Great, company info saved.");

			// write new company metadata so don't have to redo in case ctrl+c
			if let Some(mut company) = self.app.company.clone() {
				company.updated = SystemTime::now().into();
				self.app.write_company(&company)?;
			}
		}

		// regular editing
		self.menu()?;

		// if anything changed, save + show instructions
		if self.data_changed {
			if let Some(mut company) = self.app.company.clone() {
				company.updated = SystemTime::now().into();
				self.app.write_company(&company)?;
			}

			self.app.show_instructions();
		}

		Ok(())
	}

	pub fn menu(&mut self) -> Result<()> {
		if let Some(company) = self.app.company.clone() {
			let action = Select::with_theme(&self.theme)
				.with_prompt(format!("What would you like to do with {}?", company.name))
				.items(&["Add/remove jobs", "Edit company metadata", "Quit"])
				.default(0)
				.interact()?;

			match action {
				0 => self.menu_jobs()?,
				1 => {
					self.app.company = Some(self.company(&company.slug, Some(company.url))?);
					self.data_changed = true;
				}
				_ => {}
			}
		}

		Ok(())
	}

	pub fn menu_jobs(&mut self) -> Result<()> {
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
				0 => {
					let new_job = self.job(None)?;

					if let Some(company) = self.app.company.borrow_mut() {
						company.jobs.push(new_job);
						self.data_changed = true;
					}
				}
				1 if !no_jobs => {
					// if only one job, don't ask which job to edit
					let (job, index) = match &self.app.company {
						Some(company) if company.jobs.len() == 1 => (company.jobs[0].clone(), 0),
						_ => self.get_job_by_position()?,
					};

					let edited_job = self.job(Some(job))?;

					if let Some(company) = self.app.company.borrow_mut() {
						company.jobs[index] = edited_job;
						self.data_changed = true;
					}
				}
				2 => {
					let (_, index) = self.get_job_by_position()?;

					if let Some(company) = self.app.company.borrow_mut() {
						company.jobs.remove(index);
						self.data_changed = true;
					}
				}
				_ => {}
			}
		}

		Ok(())
	}

	pub fn get_job_by_position(&self) -> Result<(Job, usize)> {
		let mut m = HashMap::new();

		let mut position_names = vec![];
		if let Some(company) = &self.app.company {
			for job in company.jobs.iter() {
				let position_id = job.position.clone();
				if let Some(position) = self.app.positions.get(&position_id) {
					position_names.push(position.name.clone());
					m.insert(position.name.clone(), job.clone());
				}
			}
		}

		let index = FuzzySelect::with_theme(&self.theme)
			.with_prompt("Position")
			.default(0)
			.items(&position_names[..])
			.max_length(10)
			.interact()?;

		let position_name = position_names[index].clone();
		Ok((m[&position_name].clone(), index))
	}

	pub fn company(&self, slug: &str, maybe_url: Option<String>) -> Result<Company> {
		let mut domain = "".to_string();
		let mut url = "".to_string();

		if let Some(new_url) = maybe_url {
			if let Some((_, d, u)) = utils::parse_url(&new_url)? {
				domain = d;
				url = u;
			}
		}

		let name = {
			let mut input = Input::with_theme(&self.theme);

			input.with_prompt("Company name (eg: Example)");

			if let Some(company) = self.app.company.clone() {
				input.default(company.name);
				input.with_prompt("Company name");
			}

			input.interact()?
		};

		let at = {
			let mut input = Input::with_theme(&self.theme);

			input.with_prompt("Email address for job applications").validate_with(
				|input: &String| -> Result<(), &str> {
					if EmailAddress::from_str(input).is_ok() {
						Ok(())
					} else {
						Err("invalid email")
					}
				},
			);

			input.default(if let Some(company) = self.app.company.clone() {
				format!("{}@{domain}", company.at)
			} else {
				format!("careers@{domain}")
			});

			let email = EmailAddress::from_str(&input.interact()?)?;
			email.local_part().to_string()
		};

		let building = {
			let mut input = Input::with_theme(&self.theme);

			input.with_prompt("What are you building in 5 words or less").validate_with(
				|input: &String| -> Result<(), &str> {
					let words =
						input.split_whitespace().map(|s| s.to_string()).collect::<Vec<String>>();

					if words.is_empty() || words.len() > 5 {
						Err("invalid length")
					} else {
						Ok(())
					}
				},
			);

			if let Some(company) = self.app.company.clone() {
				input.default(company.building);
			}

			input.interact()?
		};

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

		let twitter_username: String = Input::with_theme(&self.theme)
			.with_prompt("Twitter username")
			.allow_empty(true)
			.interact()?;

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
			.default(if let Some(company) = self.app.company.clone() {
				company.founded.to_string()
			} else {
				chrono::Utc::now().year().to_string()
			})
			.interact()?
			.parse::<u16>()?;

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
			.default(if let Some(company) = self.app.company.clone() {
				company.headcount.to_string()
			} else {
				"0".to_string()
			})
			.interact()?
			.parse::<u16>()?;

		let offices = {
			let mut office_ids = HashSet::new();

			let done = "Done".to_string();

			let locations = self.app.locations.values().cloned().collect::<Vec<Item>>();
			let mut all_offices =
				locations.iter().map(|loc| loc.name.clone()).collect::<Vec<String>>();

			all_offices.sort();
			all_offices.insert(0, done);

			loop {
				let index = FuzzySelect::with_theme(&self.theme)
					.with_prompt("Any physical offices (besides remote)?")
					.default(0)
					.items(&all_offices[..])
					.max_length(10)
					.interact()?;

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

		let jobs = if let Some(company) = self.app.company.clone() { company.jobs } else { vec![] };

		Ok(Company {
			slug: slug.to_string(),
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
			updated: SystemTime::now().into(),
		})
	}

	pub fn job(&self, job: Option<Job>) -> Result<Job> {
		let position = {
			let mut ret = "".to_string();

			let positions = self.app.positions.values().cloned().collect::<Vec<Item>>();
			let mut all_positions =
				positions.iter().map(|pos| pos.name.clone()).collect::<Vec<String>>();

			all_positions.sort();

			if let Some(job) = job {
				if let Some(position) = self.app.positions.get(&job.position) {
					if let Some(index) = all_positions.iter().position(|x| *x == position.name) {
						all_positions.remove(index);
						all_positions.insert(0, position.name.clone());
					}
				}
			}

			let index = FuzzySelect::with_theme(&self.theme)
				.with_prompt("Position")
				.default(0)
				.items(&all_positions[..])
				.max_length(10)
				.interact()?;

			if let Some(position) = all_positions.get(index) {
				if let Some(position) = positions.iter().find(|pos| pos.name == *position) {
					ret = position.id.clone();
				}
			}

			ret
		};

		let level = {
			let index = Select::with_theme(&self.theme)
				.with_prompt("Level")
				.items(&["Any", "Senior", "Junior"])
				.default(0)
				.interact()?;

			match index {
				0 => Level::Any,
				1 => Level::Senior,
				_ => Level::Junior,
			}
		};

		let r#type = {
			let index = Select::with_theme(&self.theme)
				.with_prompt("Type")
				.items(&["Full-Time", "Part-Time", "Contract", "Freelance"])
				.default(0)
				.interact()?;

			match index {
				0 => Type::FullTime,
				1 => Type::PartTime,
				2 => Type::Contract,
				_ => Type::Freelance,
			}
		};

		let salary = {
			if Confirm::with_theme(&self.theme)
				.with_prompt("Is salary range transparent?")
				.interact()?
			{
				let currency = {
					let action = Select::with_theme(&self.theme)
						.with_prompt("Currency")
						.items(&["USD", "Euro"])
						.default(0)
						.interact()?;

					match action {
						0 => Currency::USD,
						_ => Currency::EUR,
					}
				};

				let range = {
					let action = Select::with_theme(&self.theme)
						.with_prompt("Pay frequency")
						.items(&["Yearly", "Monthly", "Hourly"])
						.default(0)
						.interact()?;

					match action {
						0 => Range::Yearly,
						1 => Range::Monthly,
						_ => Range::Hourly,
					}
				};

				let amount = {
					let min = Input::with_theme(&self.theme)
						.with_prompt(format!("Minimum {range} salary"))
						.default("0".to_string())
						.validate_with(|input: &String| -> Result<(), &str> {
							if input.parse::<u32>().is_ok() {
								Ok(())
							} else {
								Err("invalid salary")
							}
						})
						.interact()?
						.parse::<u32>()?;

					let max = Input::with_theme(&self.theme)
						.with_prompt(format!("Maximum {range} salary"))
						.validate_with(|input: &String| -> Result<(), &str> {
							match input.parse::<u32>() {
								Ok(value) if value < min => Err("cannot be less than min range"),
								Ok(_) => Ok(()),
								_ => Err("invalid salary"),
							}
						})
						.interact()?
						.parse::<u32>()?;

					(min, max)
				};

				Salary { amount, range, currency }
			} else {
				Salary { amount: (0, 0), range: Range::Yearly, currency: Currency::USD }
			}
		};

		let equity = {
			if Confirm::with_theme(&self.theme)
				.with_prompt("Is equity range transparent?")
				.interact()?
			{
				let min = Input::with_theme(&self.theme)
					.with_prompt("Range min percent % (eg: 0.1)")
					.default("0".to_string())
					.validate_with(|input: &String| -> Result<(), &str> {
						match input.parse::<f64>() {
							Ok(value) if value >= 0.0 => Ok(()),
							_ => Err("invalid equity"),
						}
					})
					.interact()?
					.parse::<f64>()?;

				let max = Input::with_theme(&self.theme)
					.with_prompt("Range max percent % (eg: 1.0)")
					.validate_with(|input: &String| -> Result<(), &str> {
						match input.parse::<f64>() {
							Ok(value) if value < min => Err("cannot be less than min range"),
							Ok(value) if value >= 0.0 => Ok(()),
							_ => Err("invalid equity"),
						}
					})
					.interact()?
					.parse::<f64>()?;

				(min, max)
			} else {
				(0.0, 0.0)
			}
		};

		let tech = {
			let mut tech_ids = HashSet::new();

			let done = "Done".to_string();

			let tech = self.app.tech.values().cloned().collect::<Vec<Item>>();
			let mut all_tech = tech.iter().map(|te| te.name.clone()).collect::<Vec<String>>();

			all_tech.sort();
			all_tech.insert(0, done);

			loop {
				let index = FuzzySelect::with_theme(&self.theme)
					.with_prompt("What tech will the applicant work with?")
					.default(0)
					.items(&all_tech[..])
					.max_length(10)
					.interact()?;

				if index == 0 {
					break;
				}

				if let Some(technology) = all_tech.get(index) {
					if let Some(t) = tech.iter().find(|t| t.name == *technology) {
						tech_ids.insert(t.id.clone());
					}
				}
			}

			tech_ids
		};

		let url = Input::with_theme(&self.theme)
			.with_prompt("What's the URL where applicants can apply?")
			.validate_with(|input: &String| -> Result<(), &str> {
				if Url::parse(input).is_ok() {
					Ok(())
				} else {
					Err("invalid url")
				}
			})
			.interact()?;

		Ok(Job { position, level, r#type, salary, equity, tech, url })
	}
}
