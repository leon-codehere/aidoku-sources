# Aidoku sources

A personal Aidoku source list with a patched **Asura Scans** source (v20).

Add this URL in Aidoku under **Settings → Source Lists**:

```
https://raw.githubusercontent.com/leon-codehere/aidoku-sources/main/index.min.json
```

## What's changed

Asura Scans now puts its `/comics/...` series pages behind a Cloudflare check, so v19 fails with
"Missing astro-island" when you open or refresh a series. v20 loads series details and chapter
lists from Asura's JSON API (`api.asurascans.com`) instead. See `asurascans-api-fix.patch`
for the code change against [Aidoku-Community/sources](https://github.com/Aidoku-Community/sources).

Authors and artists are not refreshed, because the API doesn't include them.

When the official source list publishes a fix with a higher version, prefer that one.

The source code comes from Aidoku-Community/sources and is dual-licensed under MIT or Apache-2.0.
