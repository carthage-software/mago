use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use pulldown_cmark::Event;
use pulldown_cmark::HeadingLevel;
use pulldown_cmark::Options;
use pulldown_cmark::Parser;
use pulldown_cmark::Tag;
use regex::Regex;
use serde::Deserialize;
use serde::Serialize;
use walkdir::WalkDir;

use crate::config::SiteConfig;

#[derive(Debug, Clone, Deserialize)]
pub struct FrontMatter {
    pub title: String,
    pub description: String,
    pub nav_order: i32,
    #[serde(default)]
    pub nav_section: String,
    #[serde(default)]
    pub nav_subsection: Option<String>,
    #[serde(default)]
    pub nav_subsubsection: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TocHeading {
    pub level: u8,
    pub id: String,
    pub title: String,
}

#[derive(Debug, Clone)]
pub struct Page {
    pub language: String,
    pub relative_markdown_path: PathBuf,
    pub logical_path: String,
    pub front_matter: FrontMatter,
    pub html: String,
    pub toc: Vec<TocHeading>,
}

impl Page {
    #[must_use]
    pub fn output_path_for_language(&self) -> PathBuf {
        if self.logical_path.is_empty() {
            PathBuf::from("index.html")
        } else {
            PathBuf::from(&self.logical_path).join("index.html")
        }
    }
}

pub struct ContentLoadResult {
    pub pages: Vec<Page>,
}

pub fn load_pages(content_root: &Path, config: &SiteConfig) -> Result<ContentLoadResult> {
    let mut pages = Vec::new();

    struct ParsedPage {
        language: String,
        relative_markdown_path: PathBuf,
        logical_path: String,
        front_matter: FrontMatter,
        markdown: String,
    }

    let mut parsed_pages = Vec::new();

    for language in &config.languages {
        let language_root = content_root.join(&language.code);
        let walker = WalkDir::new(&language_root).into_iter();

        for entry in walker.filter_map(Result::ok) {
            if !entry.file_type().is_file() {
                continue;
            }

            if entry.path().extension().and_then(|ext| ext.to_str()) != Some("md") {
                continue;
            }

            let relative_markdown_path = entry
                .path()
                .strip_prefix(&language_root)
                .with_context(|| {
                    format!(
                        "failed to strip language prefix {} from {}",
                        language_root.display(),
                        entry.path().display()
                    )
                })?
                .to_path_buf();

            let file_content = fs::read_to_string(entry.path())
                .with_context(|| format!("failed to read markdown file {}", entry.path().display()))?;
            let (front_matter, markdown) = parse_front_matter(&file_content)
                .with_context(|| format!("failed to parse front matter in {}", entry.path().display()))?;

            let logical_path = derive_logical_path(&relative_markdown_path);

            parsed_pages.push(ParsedPage {
                language: language.code.clone(),
                relative_markdown_path,
                logical_path,
                front_matter,
                markdown,
            });
        }
    }

    let mut en_slugs: HashMap<String, EnSlugs> = HashMap::new();
    for parsed in &parsed_pages {
        if parsed.language != "en" {
            continue;
        }

        let render = render_markdown(&parsed.markdown, None)?;
        en_slugs.insert(
            parsed.logical_path.clone(),
            EnSlugs { markdown: render.markdown_slugs.clone(), html: render.html_slugs.clone() },
        );

        pages.push(Page {
            language: parsed.language.clone(),
            relative_markdown_path: parsed.relative_markdown_path.clone(),
            logical_path: parsed.logical_path.clone(),
            front_matter: parsed.front_matter.clone(),
            html: render.html,
            toc: render.toc,
        });
    }

    for parsed in &parsed_pages {
        if parsed.language == "en" {
            continue;
        }

        let overrides = en_slugs.get(&parsed.logical_path);
        let render = render_markdown(&parsed.markdown, overrides)?;
        pages.push(Page {
            language: parsed.language.clone(),
            relative_markdown_path: parsed.relative_markdown_path.clone(),
            logical_path: parsed.logical_path.clone(),
            front_matter: parsed.front_matter.clone(),
            html: render.html,
            toc: render.toc,
        });
    }

    ensure_language_homepages(config, &pages)?;

    Ok(ContentLoadResult { pages })
}

fn parse_front_matter(content: &str) -> Result<(FrontMatter, String)> {
    let mut lines = content.lines();
    let first_line = lines.next().unwrap_or_default();
    if first_line.trim() != "+++" {
        anyhow::bail!("markdown file must start with +++ TOML front matter fence");
    }

    let mut front_matter_lines = Vec::new();
    let mut markdown_lines = Vec::new();
    let mut in_front_matter = true;

    for line in lines {
        if in_front_matter && line.trim() == "+++" {
            in_front_matter = false;
            continue;
        }

        if in_front_matter {
            front_matter_lines.push(line);
        } else {
            markdown_lines.push(line);
        }
    }

    if in_front_matter {
        anyhow::bail!("front matter fence is not closed");
    }

    let front_matter_raw = front_matter_lines.join("\n");
    let front_matter: FrontMatter = toml::from_str(&front_matter_raw).context("invalid front matter TOML")?;
    let markdown = markdown_lines.join("\n");

    Ok((front_matter, markdown))
}

fn derive_logical_path(relative_markdown_path: &Path) -> String {
    let is_index =
        relative_markdown_path.file_name().and_then(|name| name.to_str()).is_some_and(|name| name == "_index.md");

    if is_index {
        let Some(parent) = relative_markdown_path.parent() else {
            return String::new();
        };

        let path = parent.to_string_lossy().replace('\\', "/");
        return if path == "." { String::new() } else { path };
    }

    let mut path = relative_markdown_path.to_string_lossy().replace('\\', "/");
    if let Some(stripped) = path.strip_suffix(".md") {
        path = stripped.to_string();
    }

    path
}

struct EnSlugs {
    markdown: Vec<String>,
    html: Vec<String>,
}

struct RenderedPage {
    html: String,
    toc: Vec<TocHeading>,
    markdown_slugs: Vec<String>,
    html_slugs: Vec<String>,
}

fn render_markdown(markdown: &str, overrides: Option<&EnSlugs>) -> Result<RenderedPage> {
    let options =
        Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TABLES | Options::ENABLE_FOOTNOTES | Options::ENABLE_TASKLISTS;

    let parser = Parser::new_ext(markdown, options);
    let headings = collect_headings(parser, overrides.map(|s| s.markdown.as_slice()));

    let parser = Parser::new_ext(markdown, options);
    let mut raw_html = String::new();
    pulldown_cmark::html::push_html(&mut raw_html, parser);
    let (highlighted, html_slugs) = inject_heading_ids(&raw_html, overrides.map(|s| s.html.as_slice()))?;

    let toc = headings.iter().filter(|heading| heading.level == 2 || heading.level == 3).cloned().collect();
    let markdown_slugs = headings.into_iter().map(|heading| heading.id).collect();

    Ok(RenderedPage { html: highlighted, toc, markdown_slugs, html_slugs })
}

fn collect_headings(parser: Parser<'_>, overrides: Option<&[String]>) -> Vec<TocHeading> {
    let mut headings = Vec::new();
    let mut current_level: Option<u8> = None;
    let mut current_text = String::new();
    let mut seen_ids = HashMap::<String, usize>::new();
    let mut index = 0usize;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                current_level = Some(heading_level_to_u8(level));
                current_text.clear();
            }
            Event::End(pulldown_cmark::TagEnd::Heading(..)) => {
                if let Some(level) = current_level.take() {
                    let base =
                        overrides.and_then(|slugs| slugs.get(index).cloned()).unwrap_or_else(|| slugify(&current_text));
                    let unique_id = make_unique_slug(&base, &mut seen_ids);
                    headings.push(TocHeading { level, id: unique_id, title: current_text.trim().to_string() });
                    index += 1;
                }
            }
            Event::Text(text) | Event::Code(text) if current_level.is_some() => {
                current_text.push_str(&text);
            }
            _ => {}
        }
    }

    headings
}

fn inject_heading_ids(html: &str, overrides: Option<&[String]>) -> Result<(String, Vec<String>)> {
    let heading_regex = Regex::new(r#"(?s)<h([1-6])((?:\s[^>]*)?)>(.*?)</h[1-6]>"#).context("invalid heading regex")?;
    let strip_tags = Regex::new(r#"<[^>]*>"#).context("invalid strip-tags regex")?;
    let id_capture = Regex::new(r#"(?i)\sid\s*=\s*["']([^"']*)["']"#).context("invalid id-capture regex")?;

    let mut seen_ids = HashMap::<String, usize>::new();
    let mut output = String::with_capacity(html.len() + 256);
    let mut emitted_ids = Vec::new();
    let mut previous_end = 0;
    let mut index = 0usize;

    for capture in heading_regex.captures_iter(html) {
        let Some(full_match) = capture.get(0) else {
            continue;
        };
        output.push_str(&html[previous_end..full_match.start()]);

        let level = capture.get(1).map(|value| value.as_str()).unwrap_or("2");
        let attrs = capture.get(2).map(|value| value.as_str()).unwrap_or("");
        let inner = capture.get(3).map(|value| value.as_str()).unwrap_or_default();

        let existing_outer = id_capture.captures(attrs).and_then(|c| c.get(1));
        let existing_inner = id_capture.captures(inner).and_then(|c| c.get(1));
        if let Some(existing) = existing_outer.or(existing_inner) {
            output.push_str(full_match.as_str());
            emitted_ids.push(existing.as_str().to_string());
        } else {
            let base_slug = overrides.and_then(|slugs| slugs.get(index).cloned()).unwrap_or_else(|| {
                let plain_text = strip_tags.replace_all(inner, "");
                slugify(plain_text.trim())
            });
            let unique_id = make_unique_slug(&base_slug, &mut seen_ids);
            output.push_str(&format!(r#"<h{level}{attrs} id="{unique_id}">{inner}</h{level}>"#));
            emitted_ids.push(unique_id);
        }

        index += 1;
        previous_end = full_match.end();
    }

    output.push_str(&html[previous_end..]);
    Ok((output, emitted_ids))
}

fn slugify(input: &str) -> String {
    let mut slug = String::new();
    let mut previous_dash = false;

    for character in input.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
            previous_dash = false;
            continue;
        }

        if !previous_dash {
            slug.push('-');
            previous_dash = true;
        }
    }

    let trimmed = slug.trim_matches('-').to_string();
    if trimmed.is_empty() { "section".to_string() } else { trimmed }
}

fn make_unique_slug(base: &str, seen_ids: &mut HashMap<String, usize>) -> String {
    let count = seen_ids.entry(base.to_string()).or_insert(0usize);
    if *count == 0 {
        *count += 1;
        return base.to_string();
    }

    let slug = format!("{base}-{}", *count + 1);
    *count += 1;
    slug
}

fn heading_level_to_u8(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn ensure_language_homepages(config: &SiteConfig, pages: &[Page]) -> Result<()> {
    let mut home_languages = HashSet::new();
    for page in pages {
        if page.logical_path.is_empty() {
            home_languages.insert(page.language.as_str());
        }
    }

    for language in &config.languages {
        if !home_languages.contains(language.code.as_str()) {
            anyhow::bail!("language '{}' is missing content/{}/_index.md", language.code, language.code);
        }
    }

    Ok(())
}
