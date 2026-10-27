import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { buildManifest, platformAssets } from "./latest-json.mjs";

const VERSION = "0.4.0";
const BASE = "https://github.com/example/vile/releases/download/v0.4.0";
const SCRIPT = fileURLToPath(new URL("./latest-json.mjs", import.meta.url));
const SIGNED = [
  `vile_${VERSION}_macos_arm64.app.tar.gz`,
  `vile_${VERSION}_linux_amd64.deb`,
  `vile_${VERSION}_linux_amd64.AppImage`,
  `vile_${VERSION}_windows_x64_setup.exe`,
];

/** Fake release files: the signed assets with "<asset>.sig" next to them, and an unsigned zip. */
function fixture({ skip = [] } = {}) {
  const root = mkdtempSync(join(tmpdir(), "latest-json-"));
  const assets = join(root, "release-assets");
  const sigs = join(root, "signatures");
  mkdirSync(assets);
  mkdirSync(sigs);
  writeFileSync(join(assets, `vile_${VERSION}_macos_arm64.app.zip`), "zip");
  for (const asset of SIGNED) {
    if (skip.includes(asset)) continue;
    writeFileSync(join(assets, asset), `bytes of ${asset}`);
    writeFileSync(join(sigs, `${asset}.sig`), `  sig for ${asset}\n`);
  }
  return {
    root,
    assets,
    sigs,
    cleanup: () => rmSync(root, { recursive: true, force: true }),
  };
}

function manifest(f, extra = {}) {
  return buildManifest({
    version: VERSION,
    notes: "## Added\n\n- Updates.\n",
    baseUrl: `${BASE}/`,
    assetsDir: f.assets,
    sigsDir: f.sigs,
    now: new Date("2026-10-27T12:00:00Z"),
    ...extra,
  });
}

test("maps every platform key to its asset URL and trimmed signature", () => {
  const f = fixture();
  try {
    const m = manifest(f);
    assert.equal(m.version, VERSION);
    assert.equal(m.notes, "## Added\n\n- Updates.");
    assert.equal(m.pub_date, "2026-10-27T12:00:00.000Z");
    assert.deepEqual(
      Object.keys(m.platforms).sort(),
      platformAssets(VERSION)
        .map(([k]) => k)
        .sort(),
    );
    assert.equal(
      m.platforms["linux-x86_64-deb"].url,
      `${BASE}/vile_${VERSION}_linux_amd64.deb`,
    );
    assert.equal(
      m.platforms["linux-x86_64-deb"].signature,
      `sig for vile_${VERSION}_linux_amd64.deb`,
    );
    assert.equal(
      m.platforms["darwin-aarch64"].url,
      m.platforms["darwin-aarch64-app"].url,
    );
    assert.equal(
      m.platforms["windows-x86_64"].url,
      m.platforms["windows-x86_64-nsis"].url,
    );
  } finally {
    f.cleanup();
  }
});

test("has no plain linux key, because the installer decides the asset", () => {
  assert.equal(
    platformAssets(VERSION).some(([key]) => key === "linux-x86_64"),
    false,
  );
});

test("fails on a missing asset or signature unless --allow-missing omits it", () => {
  const f = fixture({ skip: [`vile_${VERSION}_linux_amd64.deb`] });
  try {
    assert.throws(() => manifest(f), /linux_amd64\.deb/);
    const partial = manifest(f, { allowMissing: true });
    assert.equal("linux-x86_64-deb" in partial.platforms, false);
    assert.equal("linux-x86_64-appimage" in partial.platforms, true);
  } finally {
    f.cleanup();
  }
});

test("rejects a version that is not MAJOR.MINOR.PATCH", () => {
  const f = fixture();
  try {
    assert.throws(
      () => manifest(f, { version: "v0.4.0" }),
      /MAJOR\.MINOR\.PATCH/,
    );
  } finally {
    f.cleanup();
  }
});

test("the command line prints the manifest, and exits 1 when a file is missing", () => {
  const f = fixture();
  const notes = join(f.root, "notes.md");
  writeFileSync(notes, "Notes\n");
  const run = (assets) =>
    execFileSync(
      process.execPath,
      [
        SCRIPT,
        "--version",
        VERSION,
        "--notes",
        notes,
        "--base-url",
        BASE,
        "--assets",
        assets,
        "--sigs",
        f.sigs,
      ],
      { encoding: "utf8", stdio: "pipe" },
    );
  try {
    const out = JSON.parse(run(f.assets));
    assert.equal(Object.keys(out.platforms).length, 6);
    assert.throws(
      () => run(join(f.root, "no-such-dir")),
      (error) =>
        error.status === 1 && /missing asset or signature/.test(error.stderr),
    );
  } finally {
    f.cleanup();
  }
});
