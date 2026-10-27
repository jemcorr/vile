// Writes the static updater manifest (latest.json) for one release to stdout.
//
// Usage:
//   node tools/latest-json.mjs --version 0.4.0 --notes notes.md \
//     --base-url https://github.com/owner/repo/releases/download/v0.4.0 \
//     --assets release-assets --sigs signatures [--allow-missing]
//
// Each asset must sit in --assets with its signature in --sigs as "<asset>.sig".
// A missing asset or signature fails the run unless --allow-missing is set, which
// omits that platform. Only the local test uses --allow-missing.
import { existsSync, readFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

/**
 * Each updater key and the asset it serves. Linux keys name the installer, so a
 * deb install never downloads the AppImage. The key set must match what the
 * plugin looks up: {os}-{arch}-{installer}, then {os}-{arch}.
 */
export function platformAssets(version) {
  const macos = `vile_${version}_macos_arm64.app.tar.gz`;
  const windows = `vile_${version}_windows_x64_setup.exe`;
  return [
    ["darwin-aarch64-app", macos],
    ["darwin-aarch64", macos],
    ["windows-x86_64-nsis", windows],
    ["windows-x86_64", windows],
    ["linux-x86_64-appimage", `vile_${version}_linux_amd64.AppImage`],
    ["linux-x86_64-deb", `vile_${version}_linux_amd64.deb`],
  ];
}

/** Builds the manifest object. Throws when an asset or signature is missing, unless allowMissing. */
export function buildManifest({
  version,
  notes,
  baseUrl,
  assetsDir,
  sigsDir,
  allowMissing = false,
  now = new Date(),
}) {
  if (!/^\d+\.\d+\.\d+$/.test(version)) {
    throw new Error(`version must be MAJOR.MINOR.PATCH, got "${version}"`);
  }
  const base = baseUrl.replace(/\/+$/, "");
  const platforms = {};
  const missing = [];
  for (const [key, asset] of platformAssets(version)) {
    const assetPath = join(assetsDir, asset);
    const sigPath = join(sigsDir, `${asset}.sig`);
    if (!existsSync(assetPath) || !existsSync(sigPath)) {
      missing.push(asset);
      continue;
    }
    platforms[key] = {
      signature: readFileSync(sigPath, "utf8").trim(),
      url: `${base}/${asset}`,
    };
  }
  if (missing.length > 0 && !allowMissing) {
    throw new Error(`missing asset or signature for: ${missing.join(", ")}`);
  }
  return {
    version,
    notes: notes.trim(),
    pub_date: now.toISOString(),
    platforms,
  };
}

function main(argv) {
  const { values } = parseArgs({
    args: argv,
    options: {
      version: { type: "string" },
      notes: { type: "string" },
      "base-url": { type: "string" },
      assets: { type: "string" },
      sigs: { type: "string" },
      "allow-missing": { type: "boolean", default: false },
    },
    strict: true,
  });
  for (const name of ["version", "notes", "base-url", "assets", "sigs"]) {
    if (!values[name]) throw new Error(`missing --${name}`);
  }
  const manifest = buildManifest({
    version: values.version,
    notes: readFileSync(values.notes, "utf8"),
    baseUrl: values["base-url"],
    assetsDir: values.assets,
    sigsDir: values.sigs,
    allowMissing: values["allow-missing"],
  });
  process.stdout.write(`${JSON.stringify(manifest, null, 2)}\n`);
}

if (
  process.argv[1] &&
  resolve(process.argv[1]) === fileURLToPath(import.meta.url)
) {
  try {
    main(process.argv.slice(2));
  } catch (error) {
    process.stderr.write(`latest-json: ${error.message}\n`);
    process.exit(1);
  }
}
