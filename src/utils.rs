use eyre::Result;
use slugify::slugify;
use url::Url;

pub fn parse_url(input: &str) -> Result<Option<(String, String, String)>> {
	let url = if !input.contains("://") { format!("https://{input}") } else { input.to_string() };

	let parsed_url = Url::parse(&url)?;
	if vec!["http", "https"].contains(&parsed_url.scheme()) {
		if let Some(mut domain) = parsed_url.domain().map(|s| s.to_owned()) {
			if domain.starts_with("www.") {
				domain = domain[4..].to_string();
			}

			// @TODO doesn't support domains like "example.co.uk"
			let tmp = domain.split('.').collect::<Vec<_>>();
			if tmp.len() > 2 {
				domain = format!("{}.{}", tmp[tmp.len() - 2], tmp[tmp.len() - 1])
			}

			let tmp = domain.split('.').collect::<Vec<_>>();
			let slug = slugify!(tmp[0]);

			let url = format!("{}://{}/", parsed_url.scheme(), domain);

			return Ok(Some((slug, domain, url)));
		}
	}

	Ok(None)
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::collections::HashMap;

	#[test]
	fn test_parse_url() {
		let data = HashMap::from([
			("http://example.com", ("example", "example.com", "http://example.com/")),
			("http://EXAMPLE.COM", ("example", "example.com", "http://example.com/")),
			("https://example.com", ("example", "example.com", "https://example.com/")),
			("https://example.com/abc", ("example", "example.com", "https://example.com/")),
			("https://sub.example.com/abc", ("example", "example.com", "https://example.com/")),
		]);

		for (input, output) in &data {
			assert_eq!(
				parse_url(&input).unwrap(),
				Some((output.0.to_string(), output.1.to_string(), output.2.to_string()))
			);
		}
	}
}
