use console::style;
use eyre::Result;
use glob::glob;
use std::{collections::HashMap, env, fs};

use crate::common::{Company, Item};

pub struct App {
	pub locations: HashMap<String, Item>,
	pub positions: HashMap<String, Item>,
	pub tech: HashMap<String, Item>,
	pub companies: HashMap<String, Company>,
	pub company: Option<Company>,
}

impl App {
	pub fn new() -> Result<Self> {
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

	fn read_items(&self, filename: &str) -> Result<HashMap<String, Item>> {
		let mut file_path = env::current_dir()?;
		file_path.push("data");
		file_path.push(format!("{filename}.json"));

		let file_contents: String = fs::read_to_string(file_path)?.parse()?;
		let data: Vec<Item> = serde_json::from_str(&file_contents)?;

		Ok(data.into_iter().map(|i| (i.id.clone(), i)).collect::<HashMap<String, Item>>())
	}

	fn read_companies(&self) -> Result<HashMap<String, Company>> {
		let mut companies = HashMap::new();

		for entry in glob("data/companies/**/*.json")? {
			match entry {
				Ok(path) => {
					let file_contents: String = fs::read_to_string(path.clone())?.parse()?;
					let mut company: Company = serde_json::from_str(&file_contents)?;
					company.slug = path.file_stem().unwrap().to_str().unwrap().to_string();
					companies.insert(company.slug.clone(), company);
				}
				Err(e) => return Err(e.into()),
			}
		}

		Ok(companies)
	}

	pub fn write_company(&self, company: &Company) -> Result<()> {
		let file_contents = serde_json::to_string_pretty(&company)?;

		let mut file_path = env::current_dir()?;
		file_path.push("data");
		file_path.push("companies");
		file_path.push(company.slug.chars().next().unwrap().to_string());
		file_path.push(format!("{}.json", company.slug));

		if let Some(p) = file_path.parent() {
			fs::create_dir_all(p)?;
		}
		fs::write(file_path, file_contents)?;

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
