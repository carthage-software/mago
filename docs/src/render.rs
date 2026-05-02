use std::collections::BTreeMap;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::Context;
use anyhow::Result;
use regex::Regex;
use tera::Context as TeraContext;
use tera::Tera;
use walkdir::WalkDir;

use crate::config::SiteConfig;
use crate::config::load_redirects;
use crate::content::ContentLoadResult;
use crate::content::Page;
use crate::content::load_pages;
use crate::i18n::load_languages;
use crate::i18n::t;
use crate::nav::build_language_switch;
use crate::nav::build_sidebar;
use crate::nav::build_version_switch;
use crate::nav::page_route;
use crate::nav::path_to_root;
use crate::versions::VersionsFile;

pub fn build_site(root: &Path) -> Result<()> {
    // Always refresh the rules reference page from the in-process linter
    // registry, this is cheap and keeps the docs in lock-step with the rules
    // shipping in the build we're documenting.
    crate::rules::generate(root)?;

    let config_path = root.join("config.toml");
    let content_root = root.join("content");
    let i18n_root = root.join("i18n");
    let templates_glob = root.join("templates").join("**").join("*");
    let versions_path = root.join("versions.json");
    let redirects_path = root.join("redirects.toml");
    let static_root = root.join("static");
    let dist_root = root.join("dist");

    let current_version = std::env::var("MAGO_DOCS_VERSION").unwrap_or_else(|_| "main".to_string());
    let config = SiteConfig::load(&config_path)?;
    let mut versions = VersionsFile::load(&versions_path, &current_version)?;
    let redirects = load_redirects(&redirects_path)?;
    let language_codes = config.languages.iter().map(|language| language.code.clone()).collect::<Vec<_>>();
    let i18n = load_languages(&i18n_root, &language_codes)?;
    let ContentLoadResult { mut pages } = load_pages(&content_root, &config)?;

    let known_paths = pages.iter().map(|page| page.logical_path.clone()).collect::<HashSet<_>>();
    if let Some(version) = versions.versions.iter_mut().find(|version| version.id == current_version) {
        version.paths = known_paths.iter().cloned().collect();
        version.paths.sort();
    }

    if dist_root.exists() {
        fs::remove_dir_all(&dist_root)
            .with_context(|| format!("failed to clean dist directory {}", dist_root.display()))?;
    }
    fs::create_dir_all(&dist_root).with_context(|| format!("failed to create {}", dist_root.display()))?;

    let version_root = dist_root.join(&current_version);
    fs::create_dir_all(&version_root).with_context(|| format!("failed to create {}", version_root.display()))?;

    copy_static_assets(&static_root, &version_root, &language_codes)?;

    let sponsors_source = root.join("sponsors.json");
    if sponsors_source.exists() {
        fs::copy(&sponsors_source, dist_root.join("sponsors.json"))
            .with_context(|| format!("failed to copy sponsors.json from {}", sponsors_source.display()))?;
    }

    let templates_glob = templates_glob.to_string_lossy().to_string();
    let tera = Tera::new(&templates_glob).context("failed to load templates")?;

    let language_labels =
        config.languages.iter().map(|language| (language.code.clone(), language.name.clone())).collect::<Vec<_>>();

    let benchmarks = match crate::benchmarks::fetch() {
        Ok(summary) => Some(summary),
        Err(error) => {
            tracing::warn!("Benchmark data unavailable; home stats will fall back to static text ({error}).");
            None
        }
    };
    let benchmark_tokens = build_benchmark_tokens(benchmarks.as_ref());

    pages.sort_by(|left, right| {
        left.language.cmp(&right.language).then_with(|| left.logical_path.cmp(&right.logical_path))
    });

    for page in &pages {
        let ui = i18n.get(&page.language).with_context(|| format!("missing i18n strings for {}", page.language))?;
        let p2r = path_to_root(&page.logical_path);
        let sidebar = build_sidebar(&pages, &page.language, &page.logical_path, &current_version, &p2r, &config);
        let language_switch =
            build_language_switch(&pages, &page.language, &page.logical_path, &current_version, &p2r, &language_labels);
        let version_switch =
            build_version_switch(&versions, &current_version, &page.language, &page.logical_path, &p2r);

        let mut context = TeraContext::new();
        context.insert("site_name", &config.site_name);
        context.insert("site_description", &config.site_description);
        context.insert("accent_color", &config.accent_color);
        context.insert("current_version", &current_version);
        // path_to_root is the relative prefix back to the dist tree's root ,
        // every URL emitted into HTML uses it so links work under file://
        // and http:// both. Asset paths look like `{path_to_root}main/_assets/...`.
        context.insert("path_to_root", &p2r);
        // Legacy alias kept for templates we haven't migrated yet.
        context.insert("asset_prefix", &p2r);
        context.insert("page_title", &page.front_matter.title);
        context.insert("page_description", &page.front_matter.description);
        context.insert("lang_code", &page.language);
        context.insert("lang_dir", if config.is_rtl(&page.language) { "rtl" } else { "ltr" });
        context.insert("logical_path", &page.logical_path);
        context.insert("page_url", &page_route(&p2r, &current_version, &page.language, &page.logical_path));
        context.insert(
            "edit_url",
            &format!(
                "{}/content/{}/{}",
                config.source_edit_url.trim_end_matches('/'),
                page.language,
                page.relative_markdown_path.to_string_lossy()
            ),
        );
        context.insert("sidebar", &sidebar);
        context.insert("toc", &page.toc);
        context.insert("language_switch", &language_switch);
        context.insert("version_switch", &version_switch);
        // True when the rendered page is in the project's authoritative
        // language. Translated pages are always shown with a "may be
        // outdated" banner; we trust nothing about translation freshness,
        // so we say so honestly rather than gating on a SHA we can't keep
        // accurate without contributor friction.
        context.insert("is_default_language", &(page.language == config.default_language));
        let rewritten_html = rewrite_content_urls(&page.html, &p2r, &current_version, &page.language, &language_codes)?;
        let rewritten_html = apply_benchmark_tokens(&rewritten_html, &benchmark_tokens);
        context.insert("content", &rewritten_html);
        context.insert("is_homepage", &page.logical_path.is_empty());
        context.insert("is_playground", &(page.logical_path == "playground"));
        context.insert("base_url", &config.base_url);

        let ui_strings = BTreeMap::from([
            ("on_this_page", t(ui, "on_this_page", "On this page")),
            ("search", t(ui, "search", "Search")),
            ("search_hint", t(ui, "search_hint", "Press / to search")),
            ("theme_toggle", t(ui, "theme_toggle", "Toggle theme")),
            ("language", t(ui, "language", "Language")),
            ("version", t(ui, "version", "Version")),
            ("translation_stale", t(ui, "translation_stale", "This translation may be outdated.")),
            ("untranslated_banner", t(ui, "untranslated_banner", "This page is not translated yet.")),
            ("edit_page", t(ui, "edit_page", "Edit this page")),
            ("playground_title", t(ui, "playground_title", "Playground")),
            ("playground_run", t(ui, "playground_run", "Analyze")),
            ("playground_format", t(ui, "playground_format", "Format")),
            ("playground_share", t(ui, "playground_share", "Share")),
            ("nav_guide", t(ui, "nav_guide", "Guide")),
            ("nav_faq", t(ui, "nav_faq", "FAQ")),
            ("nav_sponsor", t(ui, "nav_sponsor", "Sponsor")),
            ("nav_github", t(ui, "nav_github", "GitHub repository")),
            ("nav_discord", t(ui, "nav_discord", "Discord community")),
            // Playground chrome (rendered into HTML via Tera).
            ("pg_settings", t(ui, "pg_settings", "Settings")),
            ("pg_settings_close", t(ui, "pg_settings_close", "Close settings")),
            ("pg_pane_issues", t(ui, "pg_pane_issues", "issues")),
            ("pg_pane_settings", t(ui, "pg_pane_settings", "settings")),
            ("pg_section_analyzer", t(ui, "pg_section_analyzer", "Analyzer")),
            ("pg_section_linter", t(ui, "pg_section_linter", "Linter")),
            ("pg_section_plugins", t(ui, "pg_section_plugins", "Plugins")),
            ("pg_section_exceptions", t(ui, "pg_section_exceptions", "Exception filters")),
            ("pg_section_initializers", t(ui, "pg_section_initializers", "Class initializers")),
            ("pg_section_rules", t(ui, "pg_section_rules", "Linter rules")),
            ("pg_section_integrations", t(ui, "pg_section_integrations", "Integrations")),
            (
                "pg_hint_integrations",
                t(ui, "pg_hint_integrations", "Enable a library or framework to surface its dedicated linter rules."),
            ),
            (
                "pg_hint_rules",
                t(
                    ui,
                    "pg_hint_rules",
                    "Toggle individual rules. Rules tied to an integration only fire when their integration is enabled below.",
                ),
            ),
            ("pg_hint_analyzer", t(ui, "pg_hint_analyzer", "Toggle individual checks.")),
            ("pg_hint_plugins", t(ui, "pg_hint_plugins", "Type providers for built-in PHP and popular libraries.")),
            ("pg_hint_exceptions", t(ui, "pg_hint_exceptions", "Active because <code>checkThrows</code> is on.")),
            (
                "pg_hint_initializers",
                t(
                    ui,
                    "pg_hint_initializers",
                    "Methods treated as initializers, alongside <code>__construct</code>. Comma-separated.",
                ),
            ),
            ("pg_label_unchecked", t(ui, "pg_label_unchecked", "Unchecked exceptions")),
            ("pg_hint_unchecked", t(ui, "pg_hint_unchecked", "Class plus subclasses, comma-separated.")),
            ("pg_label_unchecked_classes", t(ui, "pg_label_unchecked_classes", "Unchecked exception classes")),
            ("pg_hint_unchecked_classes", t(ui, "pg_hint_unchecked_classes", "Exact match only, no subclasses.")),
            ("pg_search_rules", t(ui, "pg_search_rules", "Search rules.")),
            ("pg_enable_all", t(ui, "pg_enable_all", "Enable all")),
            ("pg_disable_all", t(ui, "pg_disable_all", "Disable all")),
            ("pg_rules_load", t(ui, "pg_rules_load", "Rules load with the analyzer.")),
            ("pg_loading_title", t(ui, "pg_loading_title", "Loading the analyzer.")),
            ("pg_loading_sub", t(ui, "pg_loading_sub", "First load fetches the WebAssembly module (~15 MB).")),
            // Playground dynamic strings (emitted as a JSON block; consumed by playground.js).
            ("pg_status_loading", t(ui, "pg_status_loading", "Loading the analyzer (~15 MB).")),
            ("pg_status_formatting", t(ui, "pg_status_formatting", "Formatting.")),
            ("pg_status_formatted", t(ui, "pg_status_formatted", "Formatted.")),
            ("pg_status_sharing", t(ui, "pg_status_sharing", "Sharing.")),
            ("pg_status_copied", t(ui, "pg_status_copied", "Link copied to clipboard.")),
            ("pg_status_share_url", t(ui, "pg_status_share_url", "Share URL:")),
            ("pg_err_share", t(ui, "pg_err_share", "Share failed:")),
            ("pg_err_format", t(ui, "pg_err_format", "Format error:")),
            ("pg_err_analysis", t(ui, "pg_err_analysis", "Analysis error:")),
            ("pg_err_load", t(ui, "pg_err_load", "Failed to load analyzer:")),
            ("pg_no_issues_title", t(ui, "pg_no_issues_title", "No issues found.")),
            ("pg_no_issues_sub", t(ui, "pg_no_issues_sub", "The analyzer ran clean against your code.")),
            ("pg_filtered_title", t(ui, "pg_filtered_title", "All matching issues are filtered out.")),
            ("pg_filtered_sub", t(ui, "pg_filtered_sub", "Re-enable a filter chip above to see them.")),
        ]);
        context.insert("ui", &ui_strings);

        let template_name = if page.logical_path == "playground" {
            "playground.html"
        } else if page.logical_path.is_empty() {
            "home.html"
        } else {
            "page.html"
        };
        let rendered = tera
            .render(template_name, &context)
            .with_context(|| format!("failed to render {} template", template_name))?;

        let language_output_path = version_root.join(&page.language).join(page.output_path_for_language());
        write_file(&language_output_path, &rendered)?;
    }

    write_root_redirect(&dist_root, &current_version, &config.default_language)?;
    write_version_root_redirect(&version_root, &current_version, &config.default_language)?;
    write_redirect_stubs(&dist_root, &redirects)?;
    write_sitemap(&dist_root, &config, &current_version, &pages)?;
    write_robots(&dist_root, &config, &current_version)?;
    write_versions_snapshot(&dist_root, &versions)?;
    validate_internal_links(&dist_root, &current_version)?;

    build_pagefind_index(root, &current_version)?;

    Ok(())
}

fn copy_static_assets(static_root: &Path, dist_root: &Path, language_codes: &[String]) -> Result<()> {
    let assets_root = dist_root.join("_assets");
    fs::create_dir_all(&assets_root)
        .with_context(|| format!("failed to create assets directory {}", assets_root.display()))?;

    for entry in WalkDir::new(static_root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }

        let relative_path = entry
            .path()
            .strip_prefix(static_root)
            .with_context(|| format!("failed to strip static prefix from {}", entry.path().display()))?;

        if relative_path.starts_with("playground_wasm") {
            continue;
        }

        // playground.js is shipped per-language under playground/_assets/
        // (handled below). Skip it in the global pass to avoid a duplicate
        // copy under _assets/js/.
        if relative_path == Path::new("js/playground.js") {
            continue;
        }

        // Skip the large source SVGs / PNGs that aren't actually referenced.
        // They live under static/img/ purely as project artwork and would
        // otherwise add ~24 MB to every dist build.
        let file_name = relative_path.file_name().and_then(|name| name.to_str()).unwrap_or("");
        if matches!(file_name, "icon.svg" | "banner.svg" | "logo.svg" | "logo-original.png" | "logo.png" | ".DS_Store")
        {
            continue;
        }

        let destination = assets_root.join(relative_path);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).with_context(|| format!("failed to create directory {}", parent.display()))?;
        }
        fs::copy(entry.path(), &destination).with_context(|| {
            format!("failed to copy static file {} to {}", entry.path().display(), destination.display())
        })?;
    }

    let playground_js = static_root.join("js/playground.js");
    let wasm_source_dir = static_root.join("playground_wasm");
    let wasm_js = wasm_source_dir.join("mago_wasm.js");
    let wasm_bg = wasm_source_dir.join("mago_wasm_bg.wasm");
    if !playground_js.is_file() {
        anyhow::bail!("playground entry point missing at {}", playground_js.display());
    }
    if !wasm_js.is_file() || !wasm_bg.is_file() {
        anyhow::bail!(
            "playground WASM bundle missing from {}. Run `wasm-pack build crates/wasm --target web --release --out-dir pkg-web` and copy mago_wasm.js + mago_wasm_bg.wasm there before building the docs.",
            wasm_source_dir.display(),
        );
    }

    for language in language_codes {
        let playground_assets = dist_root.join(language).join("playground").join("_assets");
        fs::create_dir_all(&playground_assets)
            .with_context(|| format!("failed to create playground assets directory {}", playground_assets.display()))?;

        fs::copy(&playground_js, playground_assets.join("playground.js")).with_context(|| {
            format!("failed to copy playground JS {}", playground_assets.join("playground.js").display())
        })?;
        fs::copy(&wasm_js, playground_assets.join("mago_wasm.js")).with_context(|| {
            format!("failed to copy playground wasm JS {}", playground_assets.join("mago_wasm.js").display())
        })?;
        fs::copy(&wasm_bg, playground_assets.join("mago_wasm_bg.wasm")).with_context(|| {
            format!("failed to copy playground wasm binary {}", playground_assets.join("mago_wasm_bg.wasm").display())
        })?;
    }

    Ok(())
}

fn write_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("failed to create directory {}", parent.display()))?;
    }
    fs::write(path, content).with_context(|| format!("failed to write {}", path.display()))
}

fn write_redirect_stubs(dist_root: &Path, redirects: &[crate::config::RedirectRule]) -> Result<()> {
    for redirect in redirects {
        let normalized_from = redirect.from.trim().trim_start_matches('/');
        let output_path = if normalized_from.ends_with(".html") {
            dist_root.join(normalized_from)
        } else if normalized_from.is_empty() {
            dist_root.join("index.html")
        } else {
            dist_root.join(normalized_from).join("index.html")
        };

        let query_suffix = if redirect.preserve_query { " + location.search" } else { "" };
        let hash_suffix = if redirect.preserve_hash { " + location.hash" } else { "" };

        let redirect_script = format!(
            r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Mago Redirect</title>
  <link rel="canonical" href="{to}">
  <meta name="robots" content="noindex">
  <script>
    var target = "{to}"{query_suffix}{hash_suffix};
    location.replace(target);
  </script>
  <meta http-equiv="refresh" content="0; url={to}">
</head>
<body>
  <p>Redirecting to <a href="{to}">{to}</a>.</p>
</body>
</html>
"#,
            to = redirect.to,
            query_suffix = query_suffix,
            hash_suffix = hash_suffix,
        );

        write_file(&output_path, &redirect_script)?;
    }

    Ok(())
}

fn write_sitemap(dist_root: &Path, config: &SiteConfig, current_version: &str, pages: &[Page]) -> Result<()> {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
"#,
    );

    for page in pages {
        // Sitemap entries are absolute production URLs; pass an empty
        // path_to_root so page_route produces a `/`-rooted path that we
        // then prefix with base_url.
        let route = page_route("/", current_version, &page.language, &page.logical_path);
        xml.push_str("  <url><loc>");
        xml.push_str(config.base_url.trim_end_matches('/'));
        xml.push_str(&route);
        xml.push_str("</loc></url>\n");
    }

    xml.push_str("</urlset>\n");
    write_file(&dist_root.join("sitemap.xml"), &xml)
}

fn write_robots(dist_root: &Path, config: &SiteConfig, current_version: &str) -> Result<()> {
    let robots = format!(
        "User-agent: *\nAllow: /\nSitemap: {}/{}{}sitemap.xml\n",
        config.base_url.trim_end_matches('/'),
        current_version,
        "/"
    );
    write_file(&dist_root.join("robots.txt"), &robots)
}

fn write_versions_snapshot(dist_root: &Path, versions: &VersionsFile) -> Result<()> {
    let content = serde_json::to_string_pretty(versions).context("failed to serialize versions snapshot")?;
    write_file(&dist_root.join("versions.json"), &content)
}

fn validate_internal_links(dist_root: &Path, current_version: &str) -> Result<()> {
    let mut known_paths = HashSet::<String>::new();
    for entry in WalkDir::new(dist_root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }

        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("html") {
            continue;
        }

        let path = entry
            .path()
            .strip_prefix(dist_root)
            .with_context(|| format!("failed to strip dist prefix from {}", entry.path().display()))?
            .to_string_lossy()
            .replace('\\', "/");
        known_paths.insert(format!("/{path}"));
    }

    let version_root_prefix = format!("/{current_version}/");
    let href_regex = Regex::new(r##"href="(/[^"?#]+)""##).context("failed to compile href regex")?;
    for entry in WalkDir::new(dist_root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("html") {
            continue;
        }

        let content = fs::read_to_string(entry.path())
            .with_context(|| format!("failed to read rendered html {}", entry.path().display()))?;
        for capture in href_regex.captures_iter(&content) {
            let Some(target) = capture.get(1) else {
                continue;
            };
            let target_str = target.as_str();
            // Skip per-version assets, the search index, and any path scoped to a
            // different deployed version (resolved at runtime against gh-pages).
            if target_str.starts_with(&format!("{}_assets/", version_root_prefix))
                || target_str.starts_with(&format!("{}pagefind/", version_root_prefix))
            {
                continue;
            }
            if target_str.starts_with("/v") && !target_str.starts_with(&version_root_prefix) {
                continue;
            }

            let normalized = if target_str.ends_with(".html") {
                target_str.to_string()
            } else {
                let trimmed = target_str.trim_end_matches('/');
                format!("{trimmed}/index.html")
            };
            if !known_paths.contains(&normalized) {
                anyhow::bail!("broken internal link '{}' found in {}", target_str, entry.path().display());
            }
        }
    }

    Ok(())
}

fn write_root_redirect(dist_root: &Path, current_version: &str, default_language: &str) -> Result<()> {
    // Relative target with explicit index.html so the stub also works when
    // the dist/ tree is opened directly via `file://`, browsers don't
    // auto-resolve a directory URL to its index.html under that scheme.
    let target = format!("{current_version}/{default_language}/index.html");
    let canonical = format!("/{current_version}/{default_language}/");
    let body = format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Mago documentation</title>
  <link rel="canonical" href="{canonical}">
  <meta name="robots" content="noindex">
  <script>location.replace("{target}" + location.search + location.hash);</script>
  <meta http-equiv="refresh" content="0; url={target}">
</head>
<body><p>Redirecting to <a href="{target}">{target}</a></p></body>
</html>
"#
    );
    write_file(&dist_root.join("index.html"), &body)
}

fn write_version_root_redirect(version_root: &Path, current_version: &str, default_language: &str) -> Result<()> {
    let target = format!("{default_language}/index.html");
    let canonical = format!("/{current_version}/{default_language}/");
    let body = format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Mago {current_version}</title>
  <link rel="canonical" href="{canonical}">
  <meta name="robots" content="noindex">
  <script>location.replace("{target}" + location.search + location.hash);</script>
  <meta http-equiv="refresh" content="0; url={target}">
</head>
<body><p>Redirecting to <a href="{target}">{target}</a></p></body>
</html>
"#
    );
    write_file(&version_root.join("index.html"), &body)
}

fn build_pagefind_index(root: &Path, current_version: &str) -> Result<()> {
    // Index only the version's own tree so the output lands at
    // `dist/<version>/pagefind/`, matching the URL prefix that search.js
    // computes (`/<version>/pagefind/...`).
    let site = format!("dist/{current_version}");
    // Exclude code blocks from search results so "variable" doesn't surface
    // every snippet that mentions it.
    let exclude = "pre, .sponsors";

    let status = Command::new("npm")
        .args(["exec", "--", "pagefind", "--site", &site, "--root-selector", "main", "--exclude-selectors", exclude])
        .current_dir(root)
        .status();

    let success = match status {
        Ok(status) if status.success() => true,
        _ => {
            let fallback = Command::new("npx")
                .args(["--yes", "pagefind", "--site", &site, "--root-selector", "main", "--exclude-selectors", exclude])
                .current_dir(root)
                .status()
                .context("failed to spawn pagefind CLI via npx")?;
            fallback.success()
        }
    };

    if !success {
        anyhow::bail!("pagefind indexing failed");
    }

    Ok(())
}

fn rewrite_content_urls(
    html: &str,
    path_to_root: &str,
    current_version: &str,
    language: &str,
    language_codes: &[String],
) -> Result<String> {
    let attribute_regex =
        Regex::new(r##"(href|src)="/([^"?#]+)([^"]*)""##).context("failed to compile content url regex")?;

    let rewritten = attribute_regex.replace_all(html, |capture: &regex::Captures<'_>| {
        let attribute = capture.get(1).map(|value| value.as_str()).unwrap_or("href");
        let path = capture.get(2).map(|value| value.as_str()).unwrap_or_default();
        let suffix = capture.get(3).map(|value| value.as_str()).unwrap_or_default();
        let path = normalize_content_path(path);
        let path = path.as_str();

        // Pagefind index lives at <version>/pagefind/ and is loaded by JS at
        // runtime, JS computes the right prefix itself, so we leave these alone.
        if path.starts_with("pagefind/") {
            return format!(r#"{attribute}="{path_to_root}{current_version}/{path}{suffix}""#);
        }

        // Author wrote a versioned path explicitly (cross-version cross-link).
        if path == current_version || path.starts_with(&format!("{current_version}/")) {
            return rebase_content_link(attribute, path_to_root, path, suffix);
        }
        if path.starts_with("v") || path == "main" || path.starts_with("main/") {
            return rebase_content_link(attribute, path_to_root, path, suffix);
        }

        // Author wrote a lang-scoped path: `/en/foo` → `/main/en/foo`.
        if language_codes.iter().any(|code| path == code || path.starts_with(&format!("{code}/"))) {
            let combined = format!("{current_version}/{path}");
            return rebase_content_link(attribute, path_to_root, &combined, suffix);
        }

        // Image referenced via `/assets/foo.png` → per-version assets dir.
        if let Some(asset_path) = path.strip_prefix("assets/") {
            return format!(r#"{attribute}="{path_to_root}{current_version}/_assets/img/{asset_path}{suffix}""#);
        }

        // Anything else is a content-relative path, prefix with version + lang.
        let combined = if path.is_empty() {
            format!("{current_version}/{language}/")
        } else {
            format!("{current_version}/{language}/{path}")
        };
        rebase_content_link(attribute, path_to_root, &combined, suffix)
    });

    Ok(rewritten.to_string())
}

/// Stitch a `path_to_root` prefix onto a logical path. Directory-style URLs
/// have their trailing slash trimmed so the deployed site reads cleanly;
/// the web server (GitHub Pages, PHP built-in) still resolves them to the
/// underlying `index.html`.
fn rebase_content_link(attribute: &str, path_to_root: &str, logical: &str, suffix: &str) -> String {
    let trimmed = logical.trim_end_matches('/');
    let target = format!("{path_to_root}{trimmed}");
    format!(r#"{attribute}="{target}{suffix}""#)
}

fn build_benchmark_tokens(summary: Option<&crate::benchmarks::BenchmarkSummary>) -> BTreeMap<String, String> {
    use crate::benchmarks::format_seconds;

    let mut tokens = BTreeMap::new();
    let placeholder = "n/a".to_string();

    let mut insert_category = |prefix: &str, cat: Option<&crate::benchmarks::CategorySummary>| {
        tokens.insert(
            format!("{prefix}_MAGO_TIME"),
            cat.map(|c| format_seconds(c.mago_seconds)).unwrap_or_else(|| placeholder.clone()),
        );
        tokens.insert(
            format!("{prefix}_FACTOR"),
            cat.map(|c| c.factor.to_string()).unwrap_or_else(|| placeholder.clone()),
        );
        let peer_a = cat.and_then(|c| c.peers.first());
        let peer_b = cat.and_then(|c| c.peers.get(1));
        tokens.insert(
            format!("{prefix}_PEER_A"),
            peer_a.map(|p| format!("{} {}", p.name, format_seconds(p.seconds))).unwrap_or_default(),
        );
        tokens.insert(
            format!("{prefix}_PEER_B"),
            peer_b.map(|p| format!("{} {}", p.name, format_seconds(p.seconds))).unwrap_or_default(),
        );
    };

    insert_category("BENCH_ANALYZER", summary.map(|s| &s.analyzer));
    insert_category("BENCH_LINTER", summary.map(|s| &s.linter));
    insert_category("BENCH_FORMATTER", summary.map(|s| &s.formatter));

    tokens.insert(
        "BENCH_PROJECT_LABEL".to_string(),
        summary.map(|s| s.project_label.to_string()).unwrap_or_else(|| "WordPress".to_string()),
    );
    tokens.insert(
        "BENCH_PROJECT_LOC".to_string(),
        summary.map(|s| s.project_loc.to_string()).unwrap_or_else(|| "7M".to_string()),
    );
    tokens.insert(
        "BENCH_AGGREGATION_DATE".to_string(),
        summary.map(|s| s.aggregation_date.clone()).unwrap_or_else(|| placeholder.clone()),
    );
    tokens.insert(
        "BENCH_MAGO_VERSION".to_string(),
        summary.map(|s| s.mago_version.clone()).unwrap_or_else(|| placeholder.clone()),
    );

    tokens
}

fn apply_benchmark_tokens(html: &str, tokens: &BTreeMap<String, String>) -> String {
    let mut output = html.to_string();
    for (key, value) in tokens {
        let needle = format!("{{{{{key}}}}}");
        if output.contains(&needle) {
            output = output.replace(&needle, value);
        }
    }
    output
}

fn normalize_content_path(path: &str) -> String {
    let mut normalized = path.to_string();
    if normalized.ends_with(".md") {
        normalized.truncate(normalized.len().saturating_sub(3));
    }

    if normalized == "index" || normalized == "_index" {
        return String::new();
    }

    if normalized.ends_with("/index") {
        normalized.truncate(normalized.len().saturating_sub("/index".len()));
    }
    if normalized.ends_with("/_index") {
        normalized.truncate(normalized.len().saturating_sub("/_index".len()));
    }

    normalized
}
