#!/usr/bin/env node
// scripts/docs-assets/capture.mjs — the capture half of `just docs-assets` (docs asset
// automation: capture -> optimize -> replace-in-place -> prune).
//
// Screenshots the canonical asset set from a LOCALLY SERVED target/docsite/ build (built by
// `scripts/docsite.sh` / `just docs-site`) via Playwright, in both light and dark themes, and
// overwrites the stable, descriptively-named files in docs/assets/ IN PLACE (never hashed —
// re-running this never accumulates duplicates).
//
// Honesty note (VR-5 / G2 — screenshots are Declared projections, never fabricated):
// target/docsite/ currently ships exactly ONE fixed light stylesheet — the same
// `:root{--fg;--bg;--accent;--dim;--code}` custom-property set is emitted by
// crates/mycelium-doc/src/emit/html.rs, crates/mycelium-doc/src/book.rs, AND
// scripts/docsite.sh's landing page, and none of the three has a `prefers-color-scheme` media
// query yet. Rather than emit a "-dark" file that is pixel-identical to "-light" (which would
// misrepresent the site as having a dark theme it doesn't), this script injects a CAPTURE-TIME
// stylesheet override (DARK_OVERRIDE_CSS below) that re-themes the page through those same
// custom properties. This is a documentation-tooling artifact — it changes nothing about the
// site's real (light-only) output. Disclosed here and in docs/guide/docsite-preview.md.
//
// Usage: node capture.mjs --base-url http://127.0.0.1:PORT --site-dir <target/docsite> --out docs/assets
// Env: MYC_PW_CHROMIUM=<path> overrides the Chromium executable Playwright launches (falls back
//      to Playwright's own managed/cached browser resolution when unset or the path is absent).

import { chromium } from "playwright-core";
import { existsSync, mkdirSync, readdirSync, statSync } from "node:fs";
import { join, relative } from "node:path";

function parseArgs(argv) {
  const out = { baseUrl: "http://127.0.0.1:8000", siteDir: "target/docsite", out: "docs/assets" };
  for (let i = 0; i < argv.length; i += 1) {
    const a = argv[i];
    if (a === "--base-url") out.baseUrl = argv[++i];
    else if (a === "--site-dir") out.siteDir = argv[++i];
    else if (a === "--out") out.out = argv[++i];
  }
  return out;
}

// Find the first file in `dir` whose name starts with `prefix` (content-derived slugs shift if a
// doc's title changes; matching by the doc's stable ID prefix rather than the full slug survives
// that). Returns null (never throws) if nothing matches — a missing target is reported, not
// silently skipped, by the caller.
function findByPrefix(dir, prefix) {
  if (!existsSync(dir)) return null;
  const hit = readdirSync(dir).find((f) => f.toLowerCase().startsWith(prefix.toLowerCase()) && f.endsWith(".html"));
  return hit ? join(dir, hit) : null;
}

// Re-themes the site's existing CSS custom properties for a capture-time-only dark rendering —
// see the file-header honesty note. Kept minimal: override the shared :root variables (covers
// body/text/links/code blocks on every page family) plus the few hardcoded literal colors that
// would otherwise look wrong against a dark background.
const DARK_OVERRIDE_CSS = `
:root{--fg:#e6e6f0;--bg:#12121a;--accent:#6fe0a0;--dim:#9494ab;--code:#1d1d29}
.browse{background:#16241b;border-color:var(--accent)}
.missing{background:#241c14;border-color:#c93}
footer{border-top-color:#2a2a38}
a.unresolved{color:#ff6b6b}
`;

async function main() {
  const { baseUrl, siteDir, out } = parseArgs(process.argv.slice(2));
  mkdirSync(out, { recursive: true });

  const pagesDir = join(siteDir, "corpus", "pages");
  const targets = [
    { name: "docsite-home", url: "/index.html" },
    { name: "nav-tree", url: "/corpus/index.html" },
    { name: "code-highlight", file: findByPrefix(pagesDir, "example-programs-reference") },
    { name: "doc-page", file: findByPrefix(pagesDir, "adr-032-") },
  ];

  const missing = targets.filter((t) => !t.url && !t.file);
  if (missing.length > 0) {
    for (const m of missing) {
      console.error(`  MISSING  ${m.name}: no matching page found under ${pagesDir} — was the site built (just docs-site)?`);
    }
    process.exit(3);
  }

  const explicitPath = process.env.MYC_PW_CHROMIUM || "/opt/pw-browsers/chromium-1194/chrome-linux/chrome";
  const launchOpts = { headless: true };
  if (existsSync(explicitPath)) {
    launchOpts.executablePath = explicitPath;
    console.log(`  using documented Chromium path: ${explicitPath}`);
  } else {
    console.log(`  documented Chromium path (${explicitPath}) not present — falling back to Playwright's managed/cached browser`);
  }

  let browser;
  try {
    browser = await chromium.launch(launchOpts);
  } catch (err) {
    console.error("  FAIL  Chromium could not launch — no screenshots were fabricated. Reason:");
    console.error(`        ${err.message}`);
    console.error("        Fix: install a browser Playwright can use, e.g.");
    console.error(`        \`npm --prefix scripts/docs-assets install && npx --prefix scripts/docs-assets playwright install chromium\``);
    process.exit(4);
  }

  const results = [];
  try {
    const page = await browser.newPage({ viewport: { width: 1280, height: 800 }, deviceScaleFactor: 1 });
    for (const t of targets) {
      const url = t.url ? `${baseUrl}${t.url}` : `${baseUrl}/${relative(siteDir, t.file)}`;
      for (const theme of ["light", "dark"]) {
        await page.emulateMedia({ colorScheme: theme });
        await page.goto(url, { waitUntil: "networkidle" });
        if (theme === "dark") {
          await page.addStyleTag({ content: DARK_OVERRIDE_CSS });
        }
        const destPath = join(out, `${t.name}-${theme}.png`);
        await page.screenshot({ path: destPath });
        const size = statSync(destPath).size;
        results.push({ name: `${t.name}-${theme}.png`, size, url });
        console.log(`  ok    ${t.name}-${theme}.png  (${size.toLocaleString()} bytes)  <- ${url}`);
      }
    }
  } finally {
    await browser.close();
  }

  console.log(`\ncaptured ${results.length} asset(s) into ${out}/`);
}

main().catch((err) => {
  console.error("  FAIL  capture.mjs crashed:", err);
  process.exit(1);
});
