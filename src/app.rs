use glob::glob;
use std::{collections::HashMap, env, fs};

use crate::common::{AppError, Company, Item};

pub struct App {
	pub locations: HashMap<String, Item>,
	pub positions: HashMap<String, Item>,
	pub tech: HashMap<String, Item>,
	pub companies: HashMap<String, Company>,
}

impl App {
	pub fn new() -> Result<Self, AppError> {
		let mut app = App {
			locations: HashMap::new(),
			positions: HashMap::new(),
			tech: HashMap::new(),
			companies: HashMap::new(),
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
}
