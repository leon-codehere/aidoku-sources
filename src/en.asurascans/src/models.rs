use crate::{
	BASE_URL,
	helpers::{get_chapter_url, get_manga_url, parse_api_date, split_names},
};
use aidoku::{
	Chapter, ContentRating, Manga, MangaStatus, Viewer,
	alloc::{
		string::{String, ToString},
		vec::Vec,
	},
	imports::{
		html::Html,
		std::{current_date, parse_date},
	},
	prelude::*,
};
use serde::{Deserialize, Serialize};

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
		manga.description = self.description.and_then(|html| {
			Html::parse_fragment(html)
				.ok()?
				.select_first("body")?
				.text()
		});
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
			chapter_number: Some(self.number),
			date_uploaded: self.published_at.as_deref().and_then(parse_api_date),
			locked,
			..Default::default()
		}
	}
}

// Dates and descriptions aren't asserted here: aidoku-test-runner doesn't
// support quoted literals in date formats, and its parsed HTML fragments have
// no `body`, unlike the app.
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
		assert_eq!(
			free.url.as_deref(),
			Some("https://asurascans.com/comics/solo-farming-in-the-tower/chapter/149")
		);
	}
}
