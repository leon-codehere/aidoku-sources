# Upstream pull request draft

Target: [Aidoku-Community/sources](https://github.com/Aidoku-Community/sources), `sources/en.asurascans`.

Before opening it:

- Set `version` in `res/source.json` to **one above upstream's**, not 22 (upstream was at 19).
- Read the diff yourself first. The repo's CONTRIBUTING.md allows AI assistance
  but expects the submitter to understand and stand behind the code.
- Rebuild `asurascans-api-fix.patch` from the fork with `git diff` if upstream has moved.

## Title

```
fix(en.asurascans): load series details and chapters from the API
```

## Description

Fixes #892 and #893 ("Missing astro-island" when opening or refreshing a series).

The `/comics/{slug}` series pages now return a Cloudflare challenge to the app,
so the `ChapterListReact` astro-island is never found. The home, browse and
ranking pages and `api.asurascans.com` are unaffected, and `get_page_list`
already reads pages from the API.

This reads series details from `GET /api/series/{slug}` and the chapter list
from `GET /api/series/{slug}/chapters` instead.

- `SeriesResponse` accepts the series response with or without a `data`
  wrapper. Both shapes were returned on the same day.
- Authors and artists come from the API's `author` and `artist` strings.
- Early access chapters are now marked as locked while `early_access_until` is
  in the future. Some early access chapters have `is_premium: false`, so they
  were previously shown as unlocked, unlike on the website.
- Details parsing moved into `ApiSeries::apply_to` and chapter parsing into
  `ApiChapter`, with tests for both response shapes, chapter locking, and the
  new `split_names` and `parse_api_date` helpers.

Not changed: the home page and the ranking listing still parse astro-islands
from HTML, which still loads. `get_page_list` still falls back to the HTML
chapter page if the API request fails.

## Checklist

- [x] Builds with no warnings or errors
- [x] `cargo fmt`
- [x] `cargo clippy` with no warnings
- [x] Tests pass
- [x] Files end with a newline; JSON uses tabs
