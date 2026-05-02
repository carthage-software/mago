use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;

use serde::Serialize;

use crate::config::SiteConfig;
use crate::content::Page;
use crate::versions::VersionsFile;

#[derive(Debug, Clone, Serialize)]
pub struct NavSection {
    pub title: String,
    pub items: Vec<NavItem>,
    pub subsections: Vec<NavSubsection>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct NavSubsection {
    pub title: String,
    pub items: Vec<NavItem>,
    pub subsubsections: Vec<NavSubsubsection>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct NavSubsubsection {
    pub title: String,
    pub items: Vec<NavItem>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct NavItem {
    pub title: String,
    pub url: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LanguageSwitchItem {
    pub code: String,
    pub name: String,
    pub url: String,
    pub active: bool,
    pub untranslated: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct VersionSwitchItem {
    pub id: String,
    pub label: String,
    pub url: String,
    pub active: bool,
}

pub fn build_sidebar(
    pages: &[Page],
    language: &str,
    current_logical_path: &str,
    current_version: &str,
    path_to_root: &str,
    config: &SiteConfig,
) -> Vec<NavSection> {
    let mut grouped = BTreeMap::<String, Vec<&Page>>::new();

    for page in pages.iter().filter(|page| page.language == language) {
        if page.front_matter.nav_section.trim().is_empty() {
            continue;
        }
        grouped.entry(page.front_matter.nav_section.clone()).or_default().push(page);
    }

    let mut sections: Vec<NavSection> = grouped
        .into_iter()
        .map(|(section_title, section_pages)| {
            let mut direct_pages: Vec<&Page> = Vec::new();
            let mut sub_groups: BTreeMap<String, Vec<&Page>> = BTreeMap::new();

            for page in section_pages {
                match &page.front_matter.nav_subsection {
                    Some(name) if !name.is_empty() => {
                        sub_groups.entry(name.clone()).or_default().push(page);
                    }
                    _ => direct_pages.push(page),
                }
            }

            sort_by_nav(&mut direct_pages);
            let items = direct_pages
                .into_iter()
                .map(|page| nav_item_for(page, language, current_version, current_logical_path, path_to_root))
                .collect::<Vec<_>>();

            let mut subsections: Vec<NavSubsection> = sub_groups
                .into_iter()
                .map(|(subsection_title, sub_pages)| {
                    let mut direct_sub: Vec<&Page> = Vec::new();
                    let mut subsub_groups: BTreeMap<String, Vec<&Page>> = BTreeMap::new();
                    for page in sub_pages {
                        match &page.front_matter.nav_subsubsection {
                            Some(name) if !name.is_empty() => {
                                subsub_groups.entry(name.clone()).or_default().push(page);
                            }
                            _ => direct_sub.push(page),
                        }
                    }

                    sort_by_nav(&mut direct_sub);
                    let sub_items: Vec<NavItem> = direct_sub
                        .into_iter()
                        .map(|page| nav_item_for(page, language, current_version, current_logical_path, path_to_root))
                        .collect();

                    let subsubsections: Vec<NavSubsubsection> = subsub_groups
                        .into_iter()
                        .map(|(subsub_title, mut subsub_pages)| {
                            sort_by_nav(&mut subsub_pages);
                            let items: Vec<NavItem> = subsub_pages
                                .into_iter()
                                .map(|page| {
                                    nav_item_for(page, language, current_version, current_logical_path, path_to_root)
                                })
                                .collect();
                            let active = items.iter().any(|item| item.active);
                            NavSubsubsection { title: subsub_title, items, is_active: active }
                        })
                        .collect();

                    let sub_active =
                        sub_items.iter().any(|item| item.active) || subsubsections.iter().any(|s| s.is_active);

                    NavSubsection { title: subsection_title, items: sub_items, subsubsections, is_active: sub_active }
                })
                .collect();

            if let Some(order) = config.nav_subsections.get(&section_title) {
                let order_index = build_index_map(order);
                subsections.sort_by(|left, right| {
                    let l = order_index.get(&left.title).copied().unwrap_or(usize::MAX);
                    let r = order_index.get(&right.title).copied().unwrap_or(usize::MAX);
                    l.cmp(&r).then_with(|| left.title.cmp(&right.title))
                });
            }

            let is_active =
                items.iter().any(|item| item.active) || subsections.iter().any(|subsection| subsection.is_active);

            NavSection { title: section_title, items, subsections, is_active }
        })
        .collect();

    let section_index = build_index_map(&config.nav_sections);
    sections.sort_by(|left, right| {
        let l = section_index.get(&left.title).copied().unwrap_or(usize::MAX);
        let r = section_index.get(&right.title).copied().unwrap_or(usize::MAX);
        l.cmp(&r).then_with(|| left.title.cmp(&right.title))
    });

    sections
}

fn sort_by_nav(pages: &mut [&Page]) {
    pages.sort_by(|left, right| {
        left.front_matter
            .nav_order
            .cmp(&right.front_matter.nav_order)
            .then_with(|| left.front_matter.title.cmp(&right.front_matter.title))
    });
}

fn nav_item_for(
    page: &Page,
    language: &str,
    current_version: &str,
    current_logical_path: &str,
    path_to_root: &str,
) -> NavItem {
    let url = page_route(path_to_root, current_version, language, &page.logical_path);
    NavItem { title: page.front_matter.title.clone(), active: page.logical_path == current_logical_path, url }
}

fn build_index_map(items: &[String]) -> HashMap<String, usize> {
    items.iter().enumerate().map(|(index, value)| (value.clone(), index)).collect()
}

pub fn build_language_switch(
    pages: &[Page],
    current_language: &str,
    current_logical_path: &str,
    current_version: &str,
    path_to_root: &str,
    languages: &[(String, String)],
) -> Vec<LanguageSwitchItem> {
    let available_paths =
        pages.iter().map(|page| (page.language.as_str(), page.logical_path.as_str())).collect::<HashSet<_>>();

    languages
        .iter()
        .map(|(language_code, language_name)| {
            let has_translation = available_paths.contains(&(language_code.as_str(), current_logical_path));
            let url = if has_translation {
                page_route(path_to_root, current_version, language_code, current_logical_path)
            } else {
                format!(
                    "{}?untranslated=1&from={}",
                    page_route(path_to_root, current_version, language_code, ""),
                    current_logical_path
                )
            };

            LanguageSwitchItem {
                code: language_code.clone(),
                name: language_name.clone(),
                url,
                active: current_language == language_code,
                untranslated: !has_translation,
            }
        })
        .collect()
}

pub fn build_version_switch(
    versions: &VersionsFile,
    current_version: &str,
    language: &str,
    logical_path: &str,
    path_to_root: &str,
) -> Vec<VersionSwitchItem> {
    let mut items: Vec<VersionSwitchItem> = Vec::with_capacity(versions.versions.len() + 1);

    // Virtual "latest" alias points at the highest semver-stable build. The
    // workflow mirrors that build under /latest/ on gh-pages so the URL is
    // real, here we only need to surface it as a switchable entry. Hidden
    // when no stable version has shipped yet.
    if let Some(latest) = pick_latest_stable(versions) {
        let has_path = latest.paths.iter().any(|path| path == logical_path);
        let url = if has_path {
            page_route(path_to_root, "latest", language, logical_path)
        } else {
            page_route(path_to_root, "latest", language, "")
        };
        items.push(VersionSwitchItem {
            id: "latest".to_string(),
            label: format!("latest ({})", latest.label),
            url,
            active: false,
        });
    }

    for version in &versions.versions {
        let has_path =
            if version.id == current_version { true } else { version.paths.iter().any(|path| path == logical_path) };

        let url = if has_path {
            page_route(path_to_root, &version.id, language, logical_path)
        } else {
            page_route(path_to_root, &version.id, language, "")
        };

        items.push(VersionSwitchItem {
            id: version.id.clone(),
            label: version.label.clone(),
            url,
            active: version.id == current_version,
        });
    }

    items
}

fn pick_latest_stable(versions: &VersionsFile) -> Option<&crate::versions::VersionInfo> {
    versions.versions.iter().filter(|v| v.stable).max_by(|a, b| semver_key(&a.id).cmp(&semver_key(&b.id)))
}

fn semver_key(id: &str) -> (u32, u32, u32) {
    let trimmed = id.trim_start_matches('v');
    let mut parts = trimmed.split('.').map(|p| p.parse::<u32>().unwrap_or(0));
    (parts.next().unwrap_or(0), parts.next().unwrap_or(0), parts.next().unwrap_or(0))
}

#[must_use]
pub fn page_route(path_to_root: &str, version: &str, language: &str, logical_path: &str) -> String {
    let mut url = String::with_capacity(path_to_root.len() + 64);
    url.push_str(path_to_root);
    url.push_str(version);
    url.push('/');
    url.push_str(language);
    url.push('/');
    if !logical_path.is_empty() {
        url.push_str(logical_path);
    }

    url
}

#[must_use]
pub fn path_to_root(logical_path: &str) -> String {
    let segments = if logical_path.is_empty() { 0 } else { logical_path.matches('/').count() + 1 };
    let depth = 2 + segments;
    "../".repeat(depth)
}
