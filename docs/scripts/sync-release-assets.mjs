import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repo =
  process.env.FOXD_RELEASE_REPO ??
  process.env.GITHUB_REPOSITORY ??
  "P8labs/foxd";
const token = process.env.GITHUB_TOKEN ?? process.env.GH_TOKEN;
const assetNames = ["foxd-linux-amd64", "foxd-linux-arm64", "foxd-linux-armv7"];

const outputDir = fileURLToPath(
  new URL("../public/downloads/latest/", import.meta.url),
);
const releaseUrl = `https://api.github.com/repos/${repo}/releases/latest`;

function githubHeaders(accept = "application/vnd.github+json") {
  const headers = {
    Accept: accept,
    "User-Agent": "foxd-docs-release-sync",
  };

  if (token) {
    headers.Authorization = `Bearer ${token}`;
  }

  return headers;
}

async function fetchJson(url) {
  const response = await fetch(url, { headers: githubHeaders() });
  if (!response.ok) {
    throw new Error(
      `Failed to fetch ${url}: ${response.status} ${response.statusText}`,
    );
  }

  return response.json();
}

async function fetchBinary(url) {
  const response = await fetch(url, {
    headers: githubHeaders("application/octet-stream"),
  });
  if (!response.ok) {
    throw new Error(
      `Failed to download ${url}: ${response.status} ${response.statusText}`,
    );
  }

  return Buffer.from(await response.arrayBuffer());
}

const release = await fetchJson(releaseUrl);
const assets = new Map(
  (release.assets ?? []).map((asset) => [asset.name, asset]),
);

await mkdir(outputDir, { recursive: true });

for (const assetName of assetNames) {
  const asset = assets.get(assetName);

  if (!asset?.url) {
    throw new Error(`Missing release asset: ${assetName}`);
  }

  const content = await fetchBinary(asset.url);
  await writeFile(path.join(outputDir, assetName), content);
}

console.log(`Synced ${assetNames.length} release assets into ${outputDir}`);
