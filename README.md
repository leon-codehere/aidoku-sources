# Aidoku sources

A personal Aidoku source list: every English source from the
[Aidoku community list](https://github.com/Aidoku-Community/sources), with a
fixed **Asura Scans** (v23) in place of the community's v19.

## Using it

This repo is **private**, so Aidoku can't read it from GitHub. To use the list,
serve this folder somewhere your phone can reach without signing in, then add
`<that address>/index.min.json` in Aidoku under **Settings → Source Lists**.

To install only the Asura fix, open `sources/en.asurascans-v23.aix` on your
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
a series. v23 loads series details and chapter lists from Asura's API
(`api.asurascans.com`) instead. It also marks early-access chapters as locked,
as the website does.

- Code: `src/en.asurascans`, a copy of the community source with the fix applied
- Upstream pull requests, split in two with patches and descriptions: `pr/`

The source code comes from Aidoku-Community/sources and is dual-licensed under
MIT or Apache-2.0.

v23 (personal, not part of the upstream PRs) also adds a **Latest Updates**
listing: the home page section gets a "See all" link that pages through every
series in update order, instead of only the 20 on Asura's home page.
