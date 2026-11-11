# Aidoku sources

A personal Aidoku source list: every English source from the
[Aidoku community list](https://github.com/Aidoku-Community/sources), with a
fixed **Asura Scans** (v24) in place of the community's v19.

## Using it

In Aidoku, go to **Settings → Source Lists**, tap **+** and add:

```
https://raw.githubusercontent.com/leon-codehere/aidoku-sources/main/index.min.json
```

This only works while the repo is public, because Aidoku can't sign in to
GitHub. Updates pushed to `main` reach the app the next time it refreshes its
source lists (GitHub can take up to 5 minutes to serve new files).

To install only the Asura fix, open `sources/en.asurascans-v24.aix` on your
phone and choose Aidoku.

## Updating

```
node update.mjs
```

This pulls the latest English sources from the community list, downloads new
versions, deletes old files and rewrites `index.json` / `index.min.json`.
Our own builds are listed in `overrides.json`. One is dropped automatically once
the community publishes the same source at an equal or higher version.

## Asura Scans fix

Asura Scans now puts its `/comics/...` series pages behind a Cloudflare check,
so the community v19 fails with "Missing astro-island" when you open or refresh
a series. v24 loads series details and chapter lists from Asura's API
(`api.asurascans.com`) instead. It also marks early-access chapters as locked,
as the website does.

- Code: `src/en.asurascans`, a copy of the community source with the fix applied
- Upstream pull requests, split in two with patches and descriptions: `pr/`

The source code comes from Aidoku-Community/sources and is dual-licensed under
MIT or Apache-2.0.

v23–v24 (personal, not part of the upstream PRs) add a **Latest Updates**
listing: the home page section gets a "See all" link that pages through every
series in update order, instead of only the 20 on Asura's home page.
The home page's Latest Updates section also shows the 100 most recently updated
series (Asura's page only renders 20), in pages of 5 with upload times.
