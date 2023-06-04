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

			return Ok(Some((slug, domain, parsed_url.to_string())));
		}
	}

	Ok(None)
}
