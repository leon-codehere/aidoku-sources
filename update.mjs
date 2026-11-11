// Rebuilds this source list from the Aidoku community list.
//
// Every community source that includes English is copied in, except where
// overrides.json has our own build. An override is dropped once the community
// publishes the same source at an equal or higher version.
//
// Usage: node update.mjs

import { existsSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";

const COMMUNITY = "https://aidoku-community.github.io/sources";
const LANGUAGE = "en";

const community = await (await fetch(`${COMMUNITY}/index.min.json`)).json();
const overrides = JSON.parse(readFileSync("overrides.json", "utf8"));

const english = community.sources.filter((source) => source.languages.includes(LANGUAGE));
const sources = [];

for (const source of english) {
	const override = overrides.find((o) => o.id === source.id);
	if (override && override.version > source.version) {
		console.log(`keep ours   ${source.id} v${override.version} (community v${source.version})`);
		sources.push(override);
		continue;
	}
	if (override) {
		console.log(`use theirs  ${source.id} v${source.version} (ours v${override.version} is outdated)`);
	}
	for (const path of [source.downloadURL, source.iconURL]) {
		if (existsSync(path)) continue;
		const response = await fetch(`${COMMUNITY}/${path}`);
		if (!response.ok) throw new Error(`${path}: HTTP ${response.status}`);
		writeFileSync(path, Buffer.from(await response.arrayBuffer()));
		console.log(`downloaded  ${path}`);
	}
	sources.push(source);
}

for (const override of overrides) {
	if (!english.some((source) => source.id === override.id)) {
		console.log(`keep ours   ${override.id} v${override.version} (not in community list)`);
		sources.push(override);
	}
}

// Delete package and icon files no longer referenced by the list.
const used = new Set(sources.flatMap((s) => [s.downloadURL, s.iconURL]));
for (const dir of ["sources", "icons"]) {
	for (const file of readdirSync(dir)) {
		if (!used.has(`${dir}/${file}`)) {
			rmSync(`${dir}/${file}`);
			console.log(`removed     ${dir}/${file}`);
		}
	}
}

sources.sort((a, b) => a.name.localeCompare(b.name));
const index = { name: "Leon's Aidoku Sources", sources };
writeFileSync("index.json", JSON.stringify(index, null, "\t") + "\n");
writeFileSync("index.min.json", JSON.stringify(index));
console.log(`${sources.length} sources written to index.min.json`);
