# Upstream pull requests

Two PRs for [Aidoku-Community/sources](https://github.com/Aidoku-Community/sources),
built on upstream commit `4ac8414`. Open them in order: PR 2 builds on PR 1.

| PR | Patch | Change | Version |
|---|---|---|---|
| 1 | `0001-...patch` | Load series details and chapters from the API | 19 → 20 |
| 2 | `0002-...patch` | Lock early access chapters | 20 → 21 |

Before opening them:

- Read both diffs and be ready to answer questions about them. CONTRIBUTING.md
  allows AI help but expects the submitter to understand the code.
- Only keep the AI line in the descriptions if it's true for you.
- If upstream has moved on, rebase and check the version numbers again.
- Note: Asura's Terms of Service forbid scraping without permission. That applies
  to the existing community source as well, not just to this change.

---

## PR 1

### Title

```
fix(en.asurascans): load series details and chapters from the API
```

### Description

Fixes #892, fixes #893.

#### Problem

Opening or refreshing any Asura Scans series fails with "Missing astro-island".
`get_manga_update` loads `/comics/{slug}` and reads the chapter list from the
`ChapterListReact` astro-island, but those series pages are now behind a
Cloudflare challenge ("Just a moment...", HTTP 403), so the island is never found.

The home, browse and ranking pages still load normally, and so does
`api.asurascans.com`, which `get_page_list` already uses for pages. That's why
browsing and reading still worked.

#### Fix

`get_manga_update` now reads from the API, like `get_page_list` does:

- details: `GET /api/series/{slug}`
- chapters: `GET /api/series/{slug}/chapters`

The responses are deserialized into typed models in `models.rs`
(`SeriesResponse`, `ApiSeries`, `ChaptersResponse`, `ApiChapter`), which also
convert them to Aidoku types, as `BookmarkItem` already does. Behaviour is kept
the same as the HTML version:

- status, content rating (Adult/Ecchi → suggestive) and viewer mapping are unchanged
- chapter keys, URLs, dates, `is_premium` locking and the `showLocked` setting are unchanged
- authors and artists now come from the API's `author`/`artist` fields

The series endpoint returned `{"data": {"series": ...}}` earlier on the same day
and `{"series": ...}` later, so `SeriesResponse` accepts both shapes. The
chapters endpoint has always returned `{"data": [...]}`.

Two small helpers were pulled out: `parse_api_date` (the existing
fractional-second handling, now shared by both date fields) and `split_names`.

#### Not changed

- The home page and ranking listing still parse HTML; those pages still load.
- `get_page_list` still falls back to the HTML chapter page if the API request
  fails. I left it to keep this PR focused, but it can be removed if preferred.

#### Testing

- Checked against the live API in a browser: series details and chapter lists
  for all 114 series on the home and browse pages parsed, 12,492 chapters total.
- An earlier version of this change, with the same API requests, was used in the
  Aidoku app on iOS: opening and refreshing series and reading chapters worked.
  **(Update this line after testing this exact version in the app.)**
- Tests: both series response shapes, chapter conversion, premium locking, and
  `split_names`. Dates and descriptions aren't asserted, because
  aidoku-test-runner doesn't support quoted literals in date formats and its
  HTML fragments have no `body`. There's a comment about this in the tests.
- `cargo fmt`, `cargo clippy` (no warnings), `aidoku package` and `aidoku verify` pass.

How I identified the cause: in a browser, requests to `/comics/{slug}` return the
Cloudflare challenge while the other pages and the API don't. That matches the
app's error. I didn't capture the app's own request.

Written with help from Claude; I've reviewed the code.

#### Checklist

- [x] Builds without warnings or errors
- [x] `cargo fmt`
- [x] `cargo clippy` with no warnings
- [x] Files end with a newline; JSON uses tabs

---

## PR 2

### Title

```
fix(en.asurascans): lock early access chapters
```

### Description

Builds on #<PR 1 number>; merge that first.

#### Problem

Early access chapters are shown as locked on the website until their early
access period ends, but the source only checks `is_premium`. Some early access
chapters have `is_premium: false`. For example, *Solo Farming in the Tower*
chapters 150 and 151 were both in early access at the same time, but only 150
had `is_premium: true`. So the source showed 151 as unlocked.

#### Fix

`ApiChapter` reads `early_access_until`. A chapter is now locked if it's premium,
**or** its early access end date is still in the future:

```rust
pub fn is_locked(&self, now: i64) -> bool {
	self.is_premium
		|| self
			.early_access_until
			.as_deref()
			.and_then(parse_api_date)
			.is_some_and(|until| until > now)
}
```

Subscribed users are unaffected, because locking still only applies when
`auth::is_subscribed()` is false, and `showLocked` still decides whether locked
chapters are listed.

#### Testing

- Checked the API data for *Solo Farming in the Tower* chapters 149–151.
- The existing tests are updated for the new `is_locked(now)` signature. The
  early access date path isn't unit tested, for the same date-format reason as
  PR 1.
- `cargo fmt`, `cargo clippy` (no warnings) and the tests pass.

Written with help from Claude; I've reviewed the code.

#### Checklist

- [x] Builds without warnings or errors
- [x] `cargo fmt`
- [x] `cargo clippy` with no warnings
- [x] Files end with a newline; JSON uses tabs
