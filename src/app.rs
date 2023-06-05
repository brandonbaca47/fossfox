use console::style;
use glob::glob;
use std::{collections::HashMap, env, fs};

use crate::common::{AppError, Company, Item};

pub struct App {
	pub locations: HashMap<String, Item>,
	pub positions: HashMap<String, Item>,
	pub tech: HashMap<String, Item>,
	pub companies: HashMap<String, Company>,
	pub company: Option<Company>,
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
		app.companies = app.read_companies()?;

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

	fn read_companies(&self) -> Result<HashMap<String, Company>, AppError> {
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

		Ok(companies)
	}

	pub fn write_company(&self, company: &Company) -> Result<(), AppError> {
		let file_contents = serde_json::to_string_pretty(&company).unwrap();

		let mut file_path = env::current_dir().unwrap();
		file_path.push("data");
		file_path.push("companies");
		file_path.push(company.slug.chars().next().unwrap().to_string());
		file_path.push(format!("{}.json", company.slug));

		if let Some(p) = file_path.parent() {
			fs::create_dir_all(p).unwrap();
		}
		fs::write(file_path, file_contents).unwrap();

		Ok(())
	}

	pub fn show_instructions(&self) {
		if let Some(company) = &self.company {
			println!(
				"\nYour data has been saved 👍\nYou can view it by doing: {}",
				style("git diff").green().bold(),
			);

			println!("\nWhen ready, commit & push so we can merge it:");
			println!("> {}", style("git add -A").green().bold());
			println!("> {}", style(format!("git commit -m \"{}\"", company.slug)).green().bold());
			println!("> {}", style("git push").green().bold());

			println!("\nMore about pull requests here:");
			println!("{}", style("https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/proposing-changes-to-your-work-with-pull-requests/creating-a-pull-request").blue().bold());
		}
	}
}
