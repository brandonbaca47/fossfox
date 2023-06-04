use chrono::{DateTime, Utc};
use console::Style;
use dialoguer::{theme::ColorfulTheme, Input};
use glob::glob;
use std::{
	collections::{HashMap, HashSet},
	env,
	error::Error,
	fs,
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
		println!("Fossfox v{} 👩‍💻", env!("CARGO_PKG_VERSION"));

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
			println!("company exists, {company:?}");
		} else {
			println!("company does not exist");
		}

		let name: String =
			Input::with_theme(&theme).with_prompt("Company name (eg: Example)").interact()?;

		let at = "".to_string();
		let building = "".to_string();
		let products = HashSet::new();
		let socials = HashSet::new();
		let offices = HashSet::new();
		let headcount = 0;
		let founded = 0;
		let jobs = vec![];

		let now = SystemTime::now();
		let now: DateTime<Utc> = now.into();

		let updated = now;

		Ok(Some(Company {
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
		}))
	}
}
