use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
	#[serde(skip_serializing)]
	pub slug: String,
	pub name: String,
	pub fqdn: String,
}

#[derive(Error, Debug)]
pub enum AppError {
	#[error("Could not find existing data")]
	CouldNotRead,
	#[error("Could not write JSON file to {0}")]
	CouldNotWrite(String),
	#[error("Unknown")]
	Unknown,
}

#[derive(Debug, Deserialize)]
pub struct Item {
	pub id: String,
	pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Company {
	#[serde(skip_serializing, skip_deserializing)]
	pub slug: String,
	pub name: String,
	pub url: String,
	pub at: String,
	pub building: String,
	pub products: HashSet<Product>,
	pub socials: HashSet<String>,
	pub offices: HashSet<String>,
	pub headcount: u16,
	pub founded: u16,
	pub jobs: Vec<Job>,
	pub updated: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Product {
	pub name: String,
	pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Job {
	pub position: String,
	pub level: Level,
	pub r#type: Type,
	pub salary: Salary,
	pub equity: (f64, f64),
	pub tech: HashSet<String>,
	pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Level {
	#[serde(rename = "junior")]
	Junior,
	#[serde(rename = "senior")]
	Senior,
	#[serde(rename = "any")]
	Any,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Type {
	#[serde(rename = "full-time")]
	FullTime,
	#[serde(rename = "part-time")]
	PartTime,
	#[serde(rename = "contract")]
	Contract,
	#[serde(rename = "freelance")]
	Freelance,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Salary {
	pub amount: (u32, u32),
	pub range: Range,
	pub currency: Currency,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Range {
	#[serde(rename = "yearly")]
	Yearly,
	#[serde(rename = "monthly")]
	Monthly,
	#[serde(rename = "hourly")]
	Hourly,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Currency {
	#[serde(rename = "usd")]
	USD,
	#[serde(rename = "eur")]
	EUR,
}
