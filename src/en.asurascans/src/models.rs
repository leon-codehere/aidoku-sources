use crate::{
	BASE_URL,
	helpers::{description_text, get_chapter_url, get_manga_url, parse_api_date, split_names},
};
use aidoku::{
	Chapter, ContentRating, Manga, MangaStatus, MangaWithChapter, Viewer,
	alloc::{
		string::{String, ToString},
		vec::Vec,
	},
	imports::std::{current_date, parse_date},
	prelude::*,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Serialize, Deserialize)]
pub struct LoginStatus {
	pub access_token: String,
	pub refresh_token: String,
	pub is_subscribed: bool,
}

impl From<RefreshResponse> for LoginStatus {
	fn from(value: RefreshResponse) -> Self {
		let is_subscribed = value
			.data
			.subscription_status
			.as_ref()
			.is_some_and(|status| status.has_subscription.unwrap_or_default())
			|| value
				.data
				.user
				.as_ref()
				.is_some_and(|user| user.has_active_subscription());
		Self {
			access_token: value.data.access_token,
			refresh_token: value.data.refresh_token.unwrap_or_default(),
			is_subscribed,
		}
	}
}

#[derive(Deserialize)]
pub struct RefreshResponse {
	pub data: RefreshResponseData,
}

#[derive(Deserialize)]
pub struct RefreshResponseData {
	pub access_token: String,
	pub refresh_token: Option<String>,
	pub subscription_status: Option<RefreshSubscriptionStatus>,
	pub user: Option<AuthUser>,
}

#[derive(Deserialize)]
pub struct RefreshSubscriptionStatus {
	pub has_subscription: Option<bool>,
}

#[derive(Deserialize)]
pub struct AuthUser {
	role: Option<String>,
	premium_until: Option<String>,
}

impl AuthUser {
	fn has_active_subscription(&self) -> bool {
		let role = self.role.as_deref().unwrap_or("user");
		matches!(role, "staff" | "moderator" | "uploader" | "admin")
			|| matches!(role, "basic" | "premium")
				&& self
					.premium_until
					.as_deref()
					.and_then(|date| parse_date(date, "yyyy-MM-dd'T'HH:mm:ss'Z'"))
					.is_some_and(|date| date > current_date())
	}
}

#[derive(Deserialize)]
pub struct BookmarkResponse {
	pub data: Vec<BookmarkItem>,
	pub meta: BookmarkResponseMeta,
}

#[derive(Deserialize)]
pub struct BookmarkResponseMeta {
	pub total: i32,
}

#[derive(Deserialize)]
pub struct BookmarkItem {
	// pub id: i32,
	series: BookmarkSeries,
}

impl From<BookmarkItem> for Manga {
	fn from(value: BookmarkItem) -> Self {
		Manga {
			key: value.series.slug,
			title: value.series.title,
			cover: Some(value.series.cover_url),
			..Default::default()
		}
	}
}

#[derive(Deserialize)]
pub struct BookmarkSeries {
	cover_url: String,
	slug: String,
	title: String,
}

/// Response of `GET {API_URL}/series/{slug}`.
///
/// The API has returned this both with and without a `data` wrapper.
#[derive(Deserialize)]
#[serde(untagged)]
pub enum SeriesResponse {
	Wrapped { data: SeriesData },
	Bare(SeriesData),
}

impl SeriesResponse {
	pub fn into_series(self) -> ApiSeries {
		match self {
			Self::Wrapped { data } | Self::Bare(data) => data.series,
		}
	}
}

#[derive(Deserialize)]
pub struct SeriesData {
	series: ApiSeries,
}

#[derive(Deserialize)]
pub struct ApiSeries {
	title: String,
	cover: Option<String>,
	description: Option<String>,
	status: Option<String>,
	#[serde(rename = "type")]
	kind: Option<String>,
	public_url: Option<String>,
	author: Option<String>,
	artist: Option<String>,
	#[serde(default)]
	genres: Vec<ApiGenre>,
}

#[derive(Deserialize)]
struct ApiGenre {
	name: String,
}

impl ApiSeries {
	/// Fills in the details of `manga` from this series.
	pub fn apply_to(self, manga: &mut Manga) {
		let tags: Vec<String> = self.genres.into_iter().map(|genre| genre.name).collect();

		manga.title = self.title;
		manga.cover = self.cover.or(manga.cover.take());
		manga.authors = self.author.as_deref().map(split_names);
		manga.artists = self.artist.as_deref().map(split_names);
		manga.description = self.description.as_deref().and_then(description_text);
		manga.url = Some(match self.public_url {
			Some(path) => format!("{BASE_URL}{path}"),
			None => get_manga_url(&manga.key),
		});
		manga.status = match self.status.as_deref() {
			Some("ongoing") => MangaStatus::Ongoing,
			Some("hiatus") => MangaStatus::Hiatus,
			Some("completed") => MangaStatus::Completed,
			Some("dropped") => MangaStatus::Cancelled,
			_ => MangaStatus::Unknown,
		};
		manga.content_rating = if tags
			.iter()
			.any(|tag| matches!(tag.as_str(), "Adult" | "Ecchi"))
		{
			ContentRating::Suggestive
		} else {
			ContentRating::Safe
		};
		manga.viewer = match self.kind.as_deref() {
			Some("mangatoon") => Viewer::RightToLeft,
			_ => Viewer::Webtoon,
		};
		manga.tags = Some(tags);
	}
}

/// Response of `GET {API_URL}/series/{slug}/chapters`.
#[derive(Deserialize)]
pub struct ChaptersResponse {
	pub data: Vec<ApiChapter>,
}

#[derive(Deserialize)]
pub struct ApiChapter {
	number: f32,
	title: Option<String>,
	#[serde(default)]
	is_premium: bool,
	early_access_until: Option<String>,
	published_at: Option<String>,
}

impl ApiChapter {
	/// Whether the chapter needs a subscription to read at time `now`.
	///
	/// Early access chapters aren't always marked as premium, so the early
	/// access end date is checked too, matching what the website shows.
	pub fn is_locked(&self, now: i64) -> bool {
		self.is_premium
			|| self
				.early_access_until
				.as_deref()
				.and_then(parse_api_date)
				.is_some_and(|until| until > now)
	}

	pub fn into_chapter(self, manga_key: &str, locked: bool) -> Chapter {
		let key = self.number.to_string();
		Chapter {
			url: Some(get_chapter_url(&key, manga_key)),
			key,
			title: self.title.filter(|title| !title.trim().is_empty()),
			chapter_number: Some(self.number),
			date_uploaded: self.published_at.as_deref().and_then(parse_api_date),
			locked,
			..Default::default()
		}
	}
}

/// Props of the home page's `LatestUpdates` astro-island.
///
/// The page only renders 20 series, but the props hold the latest chapters
/// of the 100 most recently updated ones.
#[derive(Deserialize)]
pub struct LatestUpdatesProps {
	chapters: Vec<LatestChapter>,
}

#[derive(Deserialize)]
struct LatestChapter {
	number: f32,
	published_at: Option<String>,
	comic_name: String,
	comic_slug: String,
	comic_cover: Option<String>,
}

impl LatestUpdatesProps {
	/// Parses the island's `props` attribute.
	pub fn parse(props: &str) -> Option<Self> {
		let value = serde_json::from_str(props).ok()?;
		serde_json::from_value(unwrap_astro_value(value)).ok()
	}

	/// One entry per series with its newest chapter, in the page's order.
	///
	/// Each series' chapters are listed together, newest first.
	pub fn into_entries(mut self) -> Vec<MangaWithChapter> {
		self.chapters
			.dedup_by(|next, previous| next.comic_slug == previous.comic_slug);
		self.chapters
			.into_iter()
			.map(|chapter| {
				let key = chapter.number.to_string();
				MangaWithChapter {
					chapter: Chapter {
						url: Some(get_chapter_url(&key, &chapter.comic_slug)),
						key,
						chapter_number: Some(chapter.number),
						date_uploaded: chapter.published_at.as_deref().and_then(parse_api_date),
						..Default::default()
					},
					manga: Manga {
						key: chapter.comic_slug,
						title: chapter.comic_name,
						cover: chapter.comic_cover,
						..Default::default()
					},
				}
			})
			.collect()
	}
}

/// Removes Astro's `[type, value]` wrappers from serialized island props.
///
/// Type 0 wraps a plain value or an object whose fields are wrapped too, and
/// type 1 wraps an array of wrapped values.
fn unwrap_astro_value(value: Value) -> Value {
	match value {
		Value::Array(mut pair) if pair.len() == 2 && pair[0].is_u64() => {
			match pair.pop().unwrap_or_default() {
				Value::Array(items) => {
					Value::Array(items.into_iter().map(unwrap_astro_value).collect())
				}
				Value::Object(fields) => Value::Object(
					fields
						.into_iter()
						.map(|(key, value)| (key, unwrap_astro_value(value)))
						.collect(),
				),
				other => other,
			}
		}
		Value::Object(fields) => Value::Object(
			fields
				.into_iter()
				.map(|(key, value)| (key, unwrap_astro_value(value)))
				.collect(),
		),
		other => other,
	}
}

// Dates aren't asserted here: aidoku-test-runner doesn't support quoted
// literals in date formats. Descriptions are tested in helpers.rs.
#[cfg(test)]
mod tests {
	use super::*;
	use aidoku_test::aidoku_test;

	const SERIES: &str = r#"{"id":2127,"slug":"absolute-regression","title":"Absolute Regression","cover":"https://cdn.asurascans.com/c.webp","description":"<p>A &amp; B</p>","status":"ongoing","type":"manhwa","author":"Jang Yeonghun","artist":"Studio JHS","public_url":"/comics/absolute-regression-f212c582","genres":[{"id":1,"name":"Action","slug":"action"}]}"#;

	fn series_details(json: &str) -> Manga {
		let mut manga = Manga {
			key: "absolute-regression".into(),
			..Default::default()
		};
		serde_json::from_str::<SeriesResponse>(json)
			.unwrap()
			.into_series()
			.apply_to(&mut manga);
		manga
	}

	#[aidoku_test]
	fn test_series_response_shapes() {
		let bare = series_details(&format!(r#"{{"series":{SERIES},"recommended_series":[]}}"#));
		let wrapped = series_details(&format!(r#"{{"data":{{"series":{SERIES}}}}}"#));

		for manga in [bare, wrapped] {
			assert_eq!(manga.title, "Absolute Regression");
			assert_eq!(
				manga.authors.as_deref(),
				Some(&["Jang Yeonghun".into()][..])
			);
			assert_eq!(
				manga.url.as_deref(),
				Some("https://asurascans.com/comics/absolute-regression-f212c582")
			);
			assert_eq!(manga.tags.as_deref(), Some(&["Action".into()][..]));
			assert_eq!(manga.status, MangaStatus::Ongoing);
			assert_eq!(manga.content_rating, ContentRating::Safe);
			assert_eq!(manga.viewer, Viewer::Webtoon);
		}
	}

	#[aidoku_test]
	fn test_chapter_lock() {
		let chapter = |json: &str| serde_json::from_str::<ApiChapter>(json).unwrap();
		let now = 1794371355;

		let premium = chapter(r#"{"number":150,"is_premium":true,"early_access_until":null}"#);
		assert!(premium.is_locked(now));

		let free = chapter(r#"{"number":149,"early_access_until":null}"#);
		assert!(!free.is_locked(now));
		let free = free.into_chapter("solo-farming-in-the-tower", false);
		assert_eq!(free.key, "149");
		assert_eq!(free.title, None);
		assert_eq!(
			free.url.as_deref(),
			Some("https://asurascans.com/comics/solo-farming-in-the-tower/chapter/149")
		);
	}

	#[aidoku_test]
	fn test_chapter_title() {
		let chapter = |json: &str| {
			serde_json::from_str::<ApiChapter>(json)
				.unwrap()
				.into_chapter("cyberpunk-chivalry", false)
		};
		assert_eq!(
			chapter(r#"{"number":6,"title":"Enter the Void (2)"}"#)
				.title
				.as_deref(),
			Some("Enter the Void (2)")
		);
		assert_eq!(chapter(r#"{"number":5,"title":" "}"#).title, None);
	}

	#[aidoku_test]
	fn test_latest_updates_props() {
		// Trimmed from the home page: two chapters of one series, one of another.
		let chapter = |number: u32, name: &str, slug: &str| {
			format!(
				r#"[0,{{"number":[0,{number}],"published_at":[0,"2026-11-10T22:58:51.231657Z"],"comic_name":[0,"{name}"],"comic_slug":[0,"{slug}"],"comic_cover":[0,"https://cdn.asurascans.com/c.webp"],"is_pinned":[0,false],"linked_novel":[0,{{"id":[0,56]}}]}}]"#
			)
		};
		let props = format!(
			r#"{{"chapters":[1,[{},{},{}]],"fallbackSeries":[1,[]],"mode":[0,"comics"]}}"#,
			chapter(6, "Cyberpunk Chivalry", "cyberpunk-chivalry"),
			chapter(5, "Cyberpunk Chivalry", "cyberpunk-chivalry"),
			chapter(41, "Goblin Inc", "goblin-inc"),
		);

		let entries = LatestUpdatesProps::parse(&props).unwrap().into_entries();
		assert_eq!(entries.len(), 2);
		assert_eq!(entries[0].manga.key, "cyberpunk-chivalry");
		assert_eq!(entries[0].manga.title, "Cyberpunk Chivalry");
		assert_eq!(entries[0].chapter.key, "6");
		assert_eq!(
			entries[0].chapter.url.as_deref(),
			Some("https://asurascans.com/comics/cyberpunk-chivalry/chapter/6")
		);
		assert_eq!(entries[1].manga.key, "goblin-inc");
		assert_eq!(entries[1].chapter.chapter_number, Some(41.0));

		assert!(LatestUpdatesProps::parse("not json").is_none());
	}
}
