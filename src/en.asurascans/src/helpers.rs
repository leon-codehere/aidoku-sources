use crate::BASE_URL;
use aidoku::{
	alloc::{string::String, vec::Vec},
	imports::{html::Html, std::parse_date},
	prelude::*,
};

/// Returns the ID of a manga from a URL.
pub fn get_manga_key(url: &str) -> Option<String> {
	// Asura Scans appends a random string at the end of each series slug
	// The random string is not necessary, along with the trailing '-'

	// remove query parameters
	let path = url.split('?').next().unwrap_or("");

	// find the segment after "series"
	let manga_segment = path
		.split('/')
		.skip_while(|segment| *segment != "comics")
		.nth(1)?;

	// find the last '-' and keep it in the id
	let pos = manga_segment.rfind('-')?;
	Some(manga_segment[..pos].into())
}

/// Returns the ID of a chapter from a URL.
pub fn get_chapter_key(url: &str) -> Option<String> {
	// remove query parameters
	let path = url.split('?').next().unwrap_or("");

	// find the segment after "chapter"
	let chapter_segment = path
		.split('/')
		.skip_while(|segment| *segment != "chapter")
		.nth(1)?;

	// extract only the numeric (and '.') prefix
	let end_pos = chapter_segment
		.find(|c: char| !c.is_numeric() && c != '.')
		.unwrap_or(chapter_segment.len());

	Some(chapter_segment[..end_pos].into())
}

/// Parses an API timestamp such as `2026-11-03T18:36:44.96889Z`.
///
/// The fractional seconds are dropped because their length varies.
pub fn parse_api_date(date: &str) -> Option<i64> {
	const FORMAT: &str = "yyyy-MM-dd'T'HH:mm:ss'Z'";
	match date.split_once('.') {
		Some((without_fraction, _)) => parse_date(format!("{without_fraction}Z"), FORMAT),
		None => parse_date(date, FORMAT),
	}
}

/// Converts an HTML description to plain text, keeping its paragraphs.
///
/// Descriptions use `<p>` paragraphs, `<br>` runs, empty `<p></p>` spacers,
/// or plain text. The whole description is wrapped in a `<p>` and each `<br>`
/// becomes a paragraph break, so every line of text ends up in a paragraph;
/// empty paragraphs are then dropped.
pub fn description_text(html: &str) -> Option<String> {
	let html = format!("<p>{html}</p>")
		.replace("<br>", "</p><p>")
		.replace("<br/>", "</p><p>")
		.replace("<br />", "</p><p>");
	let paragraphs: Vec<String> = Html::parse_fragment(html)
		.ok()?
		.select("p")?
		.filter_map(|p| p.text())
		.filter(|text| !text.is_empty())
		.collect();
	(!paragraphs.is_empty()).then(|| paragraphs.join("\n\n"))
}

/// Splits a comma-separated list of names, skipping the `_` placeholder.
pub fn split_names(names: &str) -> Vec<String> {
	names
		.split(',')
		.map(str::trim)
		.filter(|name| !name.is_empty() && *name != "_")
		.map(String::from)
		.collect()
}

/// Returns full URL of a manga from a manga ID.
pub fn get_manga_url(manga_id: &str) -> String {
	format!("{BASE_URL}/comics/{manga_id}")
}

/// Returns full URL of a chapter from a chapter ID and manga ID.
pub fn get_chapter_url(chapter_id: &str, manga_id: &str) -> String {
	format!("{BASE_URL}/comics/{manga_id}/chapter/{chapter_id}")
}

#[cfg(test)]
mod tests {
	use super::*;
	use aidoku_test::aidoku_test;

	#[aidoku_test]
	fn test_manga_keys() {
		assert_eq!(
			get_manga_key("https://asurascans.com/comics/swordmasters-youngest-son-cb22671f")
				.as_deref(),
			Some("swordmasters-youngest-son")
		);
		assert_eq!(
			get_manga_key(
				"https://asurascans.com/comics/swordmasters-youngest-son-cb22671f?blahblah"
			)
			.as_deref(),
			Some("swordmasters-youngest-son")
		);
		assert_eq!(
			get_manga_key(
				"https://asurascans.com/comics/swordmasters-youngest-son-cb22671f/chapter/1"
			)
			.as_deref(),
			Some("swordmasters-youngest-son")
		);
	}

	#[aidoku_test]
	fn test_chapter_keys() {
		assert_eq!(
			get_chapter_key("https://asurascans.com/comics/swordmasters-youngest-son-cb22671f"),
			None
		);
		assert_eq!(
			get_chapter_key(
				"https://asurascans.com/comics/swordmasters-youngest-son-cb22671f?blahblah"
			),
			None
		);
		assert_eq!(
			get_chapter_key(
				"https://asurascans.com/comics/swordmasters-youngest-son-cb22671f/chapter/1"
			)
			.as_deref(),
			Some("1")
		);
		assert_eq!(
			get_chapter_key("https://asurascans.com/comics/swordmasters-youngest-son/chapter/1")
				.as_deref(),
			Some("1")
		);
	}

	#[aidoku_test]
	fn test_split_names() {
		assert_eq!(split_names("Shadowless"), ["Shadowless"]);
		assert_eq!(split_names("Mon, Beonin ,"), ["Mon", "Beonin"]);
		assert!(split_names("_").is_empty());
	}

	#[aidoku_test]
	fn test_description_text() {
		assert_eq!(
			description_text("<p>A &amp; <strong>B</strong></p><p></p><p>C</p>").as_deref(),
			Some("A & B\n\nC")
		);
		assert_eq!(
			description_text("<p>[By the Studio]<br><br>Story starts.</p><p>More.</p>").as_deref(),
			Some("[By the Studio]\n\nStory starts.\n\nMore.")
		);
		assert_eq!(
			description_text("Just plain text.").as_deref(),
			Some("Just plain text.")
		);
		assert_eq!(
			description_text("<span>First line<br>Second line</span>").as_deref(),
			Some("First line\n\nSecond line")
		);
		assert_eq!(description_text("<p></p>"), None);
	}
}
