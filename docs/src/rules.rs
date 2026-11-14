//! Generate the rule reference pages from the in-process `mago-linter` and
//! `mago-twig-linter` registries.
//!
//! Walks every rule in each default `RuleRegistry` (with disabled rules
//! included so the docs surface all of them), converts each `RuleMeta` plus
//! its default level into one display shape, and writes a clippy-style
//! collapsible page per locale into `content/<lang>/tools/linter/rules.md`
//! and `content/<lang>/tools/twig-linter/rules.md`. All reader-facing
//! strings come from `i18n/<lang>.toml`.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::Context;
use anyhow::Result;
use schemars::schema_for;
use serde_json::Value;

use mago_linter::category::Category;
use mago_linter::integration::IntegrationSet;
use mago_linter::registry::RuleRegistry;
use mago_linter::requirements::RuleRequirements;
use mago_linter::rule::AnyRule;
use mago_linter::settings::RulesSettings;
use mago_linter::settings::Settings as LinterSettings;
use mago_php_version::PHPVersionRange;
use mago_reporting::Level;
use mago_twig_linter::category::Category as TwigCategory;
use mago_twig_linter::registry::RuleRegistry as TwigRuleRegistry;
use mago_twig_linter::rule::AnyRule as TwigRule;
use mago_twig_linter::settings::RulesSettings as TwigRulesSettings;
use mago_twig_linter::settings::Settings as TwigSettings;
use mago_twig_linter::version::TwigVersion;

use crate::i18n::I18nMap;
use crate::i18n::load_languages;
use crate::i18n::t;
use crate::i18n::t_format;

const SUPPORTED_LANGUAGES: &[&str] = &["en", "fr", "zh"];

/// One rule's display data, built once and shared across every locale.
struct RuleEntry {
    code: String,
    /// The rule's key in the `rules` settings table.
    config_key: String,
    description: String,
    good_example: String,
    bad_example: String,
    category: Category,
    requirements: Requirements,
    level: Level,
}

/// What a rule needs before it runs.
enum Requirements {
    Php(RuleRequirements),
    Twig(Option<TwigVersion>),
}

/// A linter whose rules get a generated reference page.
struct RulePage {
    /// The page's directory under `content/<lang>/`.
    directory: &'static str,
    nav_subsection: &'static str,
    nav_order: u32,
    /// The prefix of the page's own i18n keys: `<prefix>_page_title`, `<prefix>_intro`, and so on.
    key_prefix: &'static str,
    /// The language of the example code fences.
    example_language: &'static str,
    rules: Vec<RuleEntry>,
    default_config: Value,
    schema: Value,
}

/// Regenerate the rules pages for every supported locale.
pub fn generate(documentation_root: &Path) -> Result<()> {
    let language_codes: Vec<String> = SUPPORTED_LANGUAGES.iter().map(|s| (*s).to_string()).collect();
    let i18n_root = documentation_root.join("i18n");
    let i18n = load_languages(&i18n_root, &language_codes)?;

    for page in [php_page()?, twig_page()?] {
        write_page(documentation_root, &i18n, &page)?;
    }

    Ok(())
}

fn php_page() -> Result<RulePage> {
    let settings = LinterSettings::default();
    let registry = RuleRegistry::build(&settings, None, true);
    let rules = registry
        .rules()
        .iter()
        .map(|rule| {
            let meta = AnyRule::meta(rule);
            RuleEntry {
                code: meta.code.to_string(),
                config_key: meta.code.to_string(),
                description: meta.description.to_string(),
                good_example: meta.good_example.to_string(),
                bad_example: meta.bad_example.to_string(),
                category: meta.category,
                requirements: Requirements::Php(meta.requirements),
                level: AnyRule::default_level(rule),
            }
        })
        .collect();

    Ok(RulePage {
        directory: "tools/linter",
        nav_subsection: "Linter",
        nav_order: 70,
        key_prefix: "rules",
        example_language: "php",
        rules,
        default_config: serde_json::to_value(&settings.rules).context("failed to serialise default linter settings")?,
        schema: serde_json::to_value(schema_for!(RulesSettings))?,
    })
}

fn twig_page() -> Result<RulePage> {
    let settings = TwigSettings { version: TwigVersion::LATEST, ..TwigSettings::default() };
    let registry = TwigRuleRegistry::build(&settings, None, true);
    let rules = registry
        .rules()
        .iter()
        .map(|rule| {
            let meta = TwigRule::meta(rule);
            RuleEntry {
                code: meta.code.to_string(),
                config_key: meta.code.trim_start_matches("twig-").to_string(),
                description: meta.description.to_string(),
                good_example: meta.good_example.to_string(),
                bad_example: meta.bad_example.to_string(),
                category: twig_category(meta.category),
                requirements: Requirements::Twig(meta.requirements.minimum_twig_version()),
                level: TwigRule::default_level(rule),
            }
        })
        .collect();

    Ok(RulePage {
        directory: "tools/twig-linter",
        nav_subsection: "Twig Linter",
        nav_order: 45,
        key_prefix: "twig_rules",
        example_language: "twig",
        rules,
        default_config: serde_json::to_value(TwigRulesSettings::default())
            .context("failed to serialise default Twig linter settings")?,
        schema: serde_json::to_value(schema_for!(TwigRulesSettings))?,
    })
}

fn twig_category(category: TwigCategory) -> Category {
    match category {
        TwigCategory::Clarity => Category::Clarity,
        TwigCategory::BestPractices => Category::BestPractices,
        TwigCategory::Consistency => Category::Consistency,
        TwigCategory::Deprecation => Category::Deprecation,
        TwigCategory::Maintainability => Category::Maintainability,
        TwigCategory::Redundancy => Category::Redundancy,
        TwigCategory::Security => Category::Security,
        TwigCategory::Safety => Category::Safety,
        TwigCategory::Correctness => Category::Correctness,
    }
}

fn write_page(documentation_root: &Path, i18n: &BTreeMap<String, I18nMap>, page: &RulePage) -> Result<()> {
    tracing::info!(
        "Regenerating {} rules ({} rules across {} categories).",
        page.directory,
        page.rules.len(),
        distinct_categories(&page.rules).len()
    );

    for language in SUPPORTED_LANGUAGES {
        let lang_root = documentation_root.join("content").join(language).join(page.directory);
        let new_page = lang_root.join("rules.md");
        let stale_dir = lang_root.join("rules");
        let stale_overview = lang_root.join("rules-and-categories.md");

        // Wipe any previous per-category layout entirely.
        if stale_dir.exists() {
            fs::remove_dir_all(&stale_dir).with_context(|| format!("failed to remove {}", stale_dir.display()))?;
        }

        if stale_overview.exists() {
            fs::remove_file(&stale_overview)
                .with_context(|| format!("failed to remove {}", stale_overview.display()))?;
        }

        let strings = i18n.get(*language).with_context(|| format!("missing i18n strings for {language}"))?;

        let body = render_all_rules(strings, page);
        let markdown = wrap_with_front_matter(strings, page, &body);
        if let Some(parent) = new_page.parent() {
            fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
        }

        fs::write(&new_page, &markdown).with_context(|| format!("failed to write {}", new_page.display()))?;
    }

    tracing::info!("Wrote {}/rules.md for every locale.", page.directory);
    Ok(())
}

/// Looks up `<prefix>_<suffix>`, then the shared `rules_<suffix>`, then `fallback`.
fn page_text(strings: &I18nMap, page: &RulePage, suffix: &str, fallback: &str) -> String {
    t(strings, &format!("{}_{suffix}", page.key_prefix), &t(strings, &format!("rules_{suffix}"), fallback))
}

fn level_rank(level: Level) -> u8 {
    match level {
        Level::Error => 0,
        Level::Warning => 1,
        Level::Note => 2,
        Level::Help => 3,
    }
}

fn level_label(level: Level) -> &'static str {
    match level {
        Level::Error => "error",
        Level::Warning => "warning",
        Level::Note => "note",
        Level::Help => "help",
    }
}

fn distinct_categories(rules: &[RuleEntry]) -> Vec<Category> {
    let mut seen: Vec<Category> = rules.iter().map(|rule| rule.category).collect();
    seen.sort();
    seen.dedup();
    seen
}

fn wrap_with_front_matter(strings: &I18nMap, page: &RulePage, body: &str) -> String {
    let title = page_text(strings, page, "page_title", "Rules");
    let description = page_text(
        strings,
        page,
        "page_description",
        "The full reference of every linter rule, sorted by severity. Click any rule to expand its description, examples, and configuration.",
    );

    let mut out = String::new();
    out.push_str("+++\n");
    out.push_str(&format!("title = \"{title}\"\n"));
    out.push_str(&format!("description = \"{description}\"\n"));
    out.push_str(&format!("nav_order = {}\n", page.nav_order));
    out.push_str("nav_section = \"Tools\"\n");
    out.push_str(&format!("nav_subsection = \"{}\"\n", page.nav_subsection));
    out.push_str("+++\n");
    out.push_str(body);
    out
}

fn render_all_rules(strings: &I18nMap, page: &RulePage) -> String {
    let rules = &page.rules;
    let mut out = String::new();

    let intro = page_text(
        strings,
        page,
        "intro",
        "Mago's linter ships {rules} rules across {categories} categories. Click any rule to expand its description, requirements, default configuration, and examples.",
    )
    .replace("{rules}", &rules.len().to_string())
    .replace("{categories}", &distinct_categories(rules).len().to_string());
    out.push_str(&intro);
    out.push_str("\n\n");

    let mut by_category: BTreeMap<Category, Vec<&RuleEntry>> = BTreeMap::new();
    for rule in rules {
        by_category.entry(rule.category).or_default().push(rule);
    }

    out.push_str(&format!(
        "<div class=\"rule-index\" role=\"navigation\" aria-label=\"{}\">",
        t(strings, "rules_categories_aria", "Rule categories"),
    ));

    for (category, rules_in_category) in &by_category {
        let display = category_display(strings, *category);
        let slug = category_slug(*category);
        let blurb = category_blurb(strings, page, *category);
        out.push_str(&format!(
            "<a class=\"rule-index__item\" href=\"#{slug}\"><span class=\"rule-index__name\">{display}</span><span class=\"rule-index__count\">{count_label}</span><span class=\"rule-index__blurb\">{blurb}</span></a>",
            count_label = rule_count_label(strings, rules_in_category.len()),
        ));
    }

    out.push_str("</div>\n\n");

    out.push_str(&render_integration_index(strings, rules));

    for (category, mut rules_in_category) in by_category {
        rules_in_category.sort_by(|left, right| {
            level_rank(left.level).cmp(&level_rank(right.level)).then_with(|| left.code.cmp(&right.code))
        });

        let display = category_display(strings, category);
        let slug = category_slug(category);
        out.push_str(&format!("<h2 id=\"{slug}\">{display}</h2>\n\n"));
        out.push_str(&category_blurb(strings, page, category));
        out.push_str("\n\n");
        out.push_str("<div class=\"rule-list\">\n\n");
        for rule in rules_in_category {
            out.push_str(&render_rule(strings, page, rule));
            out.push('\n');
        }

        out.push_str("</div>\n\n");
    }

    out
}

fn render_integration_index(strings: &I18nMap, rules: &[RuleEntry]) -> String {
    let mut by_integration: BTreeMap<String, Vec<&RuleEntry>> = BTreeMap::new();
    for rule in rules {
        let Requirements::Php(requirements) = &rule.requirements else {
            continue;
        };

        for set in requirements.required_integrations() {
            for integration in integrations_in_set(set) {
                by_integration.entry(integration.to_string()).or_default().push(rule);
            }
        }
    }

    if by_integration.is_empty() {
        return String::new();
    }

    let mut out = String::new();
    out.push_str(&format!(
        "<h2 id=\"integration-specific-rules\">{}</h2>\n\n",
        t(strings, "rules_integration_title", "Integration-specific rules"),
    ));
    out.push_str(&t(
        strings,
        "rules_integration_blurb",
        "Some rules only fire when Mago detects a particular library or framework. Each rule is linked to its full description in the section above.",
    ));
    out.push_str("\n\n");

    for (integration, mut integration_rules) in by_integration {
        integration_rules.sort_by(|left, right| left.code.cmp(&right.code));
        let slug = integration_slug(&integration);
        out.push_str(&format!("<h3 id=\"{slug}\">{integration}</h3>\n\n"));
        for rule in integration_rules {
            out.push_str(&format!("- [`{}`](#{})\n", rule.code, rule.code));
        }
        out.push('\n');
    }

    out
}

fn integrations_in_set(set: IntegrationSet) -> Vec<mago_linter::integration::Integration> {
    use mago_linter::integration::Integration;
    const ALL: &[Integration] = &[
        Integration::Psl,
        Integration::Guzzle,
        Integration::Monolog,
        Integration::Carbon,
        Integration::Amphp,
        Integration::ReactPHP,
        Integration::Symfony,
        Integration::Laravel,
        Integration::Tempest,
        Integration::Neutomic,
        Integration::Spiral,
        Integration::CakePHP,
        Integration::Yii,
        Integration::Laminas,
        Integration::Cycle,
        Integration::Doctrine,
        Integration::WordPress,
        Integration::Drupal,
        Integration::Magento,
        Integration::PHPUnit,
        Integration::Pest,
        Integration::Behat,
        Integration::Codeception,
        Integration::PHPSpec,
    ];
    ALL.iter().copied().filter(|integration| set.contains(*integration)).collect()
}

fn integration_slug(name: &str) -> String {
    format!("integration-{}", name.to_ascii_lowercase())
}

fn category_display(strings: &I18nMap, category: Category) -> String {
    t(strings, &format!("rules_category_{}_name", category_variant(category)), category.as_str())
}

fn category_blurb(strings: &I18nMap, page: &RulePage, category: Category) -> String {
    page_text(strings, page, &format!("category_{}_blurb", category_variant(category)), "")
}

fn category_variant(category: Category) -> &'static str {
    match category {
        Category::Clarity => "Clarity",
        Category::BestPractices => "BestPractices",
        Category::Consistency => "Consistency",
        Category::Deprecation => "Deprecation",
        Category::Maintainability => "Maintainability",
        Category::Redundancy => "Redundancy",
        Category::Security => "Security",
        Category::Safety => "Safety",
        Category::Correctness => "Correctness",
    }
}

fn rule_count_label(strings: &I18nMap, count: usize) -> String {
    let key = if count == 1 { "rules_count_one" } else { "rules_count_many" };
    let fallback = if count == 1 { "{count} rule" } else { "{count} rules" };
    t_format(strings, key, fallback, &[("count", &count.to_string())])
}

fn category_slug(category: Category) -> String {
    category_variant(category).to_lowercase()
}

fn strip_em_dashes(input: &str) -> String {
    input.replace(" \u{2014} ", ", ").replace('\u{2014}', ",")
}

fn render_rule(strings: &I18nMap, page: &RulePage, rule: &RuleEntry) -> String {
    let mut out = String::new();
    let level = level_label(rule.level);

    let aria = t_format(strings, "rules_permalink_aria", "Permalink to {code}", &[("code", &rule.code)]);

    out.push_str(&format!("<details class=\"rule\" name=\"rule\" id=\"{}\">\n", rule.code));
    out.push_str(&format!(
        "<summary><code class=\"rule__code\">{code}</code><a class=\"rule__anchor\" href=\"#{code}\" aria-label=\"{aria}\">¶</a><span class=\"rule__level rule__level--{level}\">{level}</span></summary>\n\n",
        code = rule.code,
    ));

    out.push_str("<div class=\"rule__body\">\n\n");

    let description = strip_em_dashes(rule.description.trim());
    if !description.is_empty() {
        out.push_str(&description);
        out.push_str("\n\n");
    }

    let req = match &rule.requirements {
        Requirements::Php(requirements) => render_requirements(strings, requirements),
        Requirements::Twig(version) => render_twig_requirements(strings, *version),
    };
    if !req.is_empty() {
        out.push_str(&req);
    }

    let has_top = !description.is_empty() || !req.is_empty();

    let good = rule.good_example.trim();
    let bad = rule.bad_example.trim();
    let has_examples = !good.is_empty() || !bad.is_empty();
    if has_examples {
        if has_top {
            out.push_str("<hr class=\"rule__separator\">\n\n");
        }

        out.push_str("<div class=\"rule-examples\">\n\n");

        if !bad.is_empty() {
            out.push_str("<div class=\"rule-example rule-example--bad\">\n");
            out.push_str(&format!(
                "<div class=\"rule-example__label\">{}</div>\n\n```{}\n",
                t(strings, "rules_avoid_label", "Avoid"),
                page.example_language,
            ));
            out.push_str(bad);
            out.push_str("\n```\n\n</div>\n\n");
        }

        if !good.is_empty() {
            out.push_str("<div class=\"rule-example rule-example--good\">\n");
            out.push_str(&format!(
                "<div class=\"rule-example__label\">{}</div>\n\n```{}\n",
                t(strings, "rules_prefer_label", "Prefer"),
                page.example_language,
            ));
            out.push_str(good);
            out.push_str("\n```\n\n</div>\n\n");
        }

        out.push_str("</div>\n\n");
    }

    if let Some(table) = render_config_table(strings, &rule.config_key, &page.default_config, &page.schema) {
        if has_top || has_examples {
            out.push_str("<hr class=\"rule__separator\">\n\n");
        }
        out.push_str(&table);
    }

    out.push_str("</div>\n\n");
    out.push_str("</details>\n");
    out
}

fn render_requirements(strings: &I18nMap, req: &RuleRequirements) -> String {
    let mut sentences: Vec<String> = Vec::new();

    for range in req.php_version_ranges() {
        let (min, max) = php_range_bounds(&range);
        match (min, max) {
            (Some(min), Some(max)) => sentences.push(t_format(
                strings,
                "rules_php_range",
                "This rule requires PHP version <code>{min}</code> through <code>{max}</code>.",
                &[("min", &min), ("max", &max)],
            )),
            (Some(min), None) => sentences.push(t_format(
                strings,
                "rules_php_min",
                "This rule requires PHP version <code>{min}</code> or newer.",
                &[("min", &min)],
            )),
            (None, Some(max)) => sentences.push(t_format(
                strings,
                "rules_php_max",
                "This rule requires a PHP version older than <code>{max}</code>.",
                &[("max", &max)],
            )),
            (None, None) => {}
        }
    }

    let integration_groups: Vec<Vec<String>> = req
        .required_integrations()
        .into_iter()
        .map(|set| integrations_in_set(set).iter().map(|i| i.to_string()).collect::<Vec<_>>())
        .filter(|group| !group.is_empty())
        .collect();

    if integration_groups.len() == 1 {
        let group = &integration_groups[0];
        let names: Vec<String> = group.iter().map(|i| integration_link_html(i)).collect();
        let joined = join_with_and(strings, &names);
        let key = if names.len() == 1 { "rules_integration_one" } else { "rules_integration_many" };
        let fallback = if names.len() == 1 {
            "This rule requires the {names} integration to be enabled."
        } else {
            "This rule requires the {names} integrations to be enabled."
        };
        sentences.push(t_format(strings, key, fallback, &[("names", &joined)]));
    } else if integration_groups.len() > 1 {
        let alternatives: Vec<String> = integration_groups
            .iter()
            .map(|group| {
                let names: Vec<String> = group.iter().map(|i| integration_link_html(i)).collect();
                join_with_and(strings, &names)
            })
            .collect();
        let separator = t(strings, "rules_integration_alternative_separator", "; or ");
        let joined = alternatives.join(&separator);
        sentences.push(t_format(
            strings,
            "rules_integration_alternatives",
            "This rule requires one of the following integration sets to be enabled: {alternatives}.",
            &[("alternatives", &joined)],
        ));
    }

    if sentences.is_empty() {
        return String::new();
    }

    let mut out = String::new();
    for sentence in sentences {
        out.push_str(&format!("<blockquote class=\"rule-requirement\">{sentence}</blockquote>\n\n"));
    }
    out
}

fn render_twig_requirements(strings: &I18nMap, version: Option<TwigVersion>) -> String {
    let Some(version) = version else {
        return String::new();
    };

    let sentence = t_format(
        strings,
        "twig_rules_twig_min",
        "This rule needs the minimum Twig version, <code>[twig.linter] version</code>, to be <code>{min}</code> or newer.",
        &[("min", &version.to_string())],
    );

    format!("<blockquote class=\"rule-requirement\">{sentence}</blockquote>\n\n")
}

/// Pull the textual lower/upper bound from a PHPVersionRange. Returns
/// `None` for unbounded sides (i.e. "any older" / "any newer").
fn php_range_bounds(range: &PHPVersionRange) -> (Option<String>, Option<String>) {
    let value = serde_json::to_value(range).unwrap_or(Value::Null);
    let min = value.get("min").and_then(version_as_string);
    let max = value.get("max").and_then(version_as_string);
    (min, max)
}

fn version_as_string(value: &Value) -> Option<String> {
    if value.is_null() {
        return None;
    }
    if let Some(s) = value.as_str() {
        return Some(s.to_string());
    }
    if let Some(parts) = value.as_array() {
        let nums: Vec<String> = parts.iter().filter_map(|v| v.as_u64().map(|n| n.to_string())).collect();
        if !nums.is_empty() {
            return Some(nums.join("."));
        }
    }
    if let Some(obj) = value.as_object() {
        let major = obj.get("major").and_then(Value::as_u64);
        let minor = obj.get("minor").and_then(Value::as_u64);
        let patch = obj.get("patch").and_then(Value::as_u64);
        if let (Some(major), Some(minor)) = (major, minor) {
            return Some(match patch {
                Some(p) => format!("{major}.{minor}.{p}"),
                None => format!("{major}.{minor}"),
            });
        }
    }
    None
}

fn integration_link_html(name: &str) -> String {
    let slug = integration_slug(name);
    format!("<a href=\"#{slug}\"><code>{name}</code></a>")
}

fn join_with_and(strings: &I18nMap, items: &[String]) -> String {
    match items {
        [] => String::new(),
        [single] => single.clone(),
        [first, second] => {
            t_format(strings, "rules_join_and_two", "{first} and {second}", &[("first", first), ("second", second)])
        }
        _ => {
            let last = items.last().expect("non-empty slice");
            let head = &items[..items.len() - 1];
            let separator = t(strings, "rules_join_separator", ", ");
            t_format(
                strings,
                "rules_join_and_many",
                "{head}, and {last}",
                &[("head", &head.join(&separator)), ("last", last)],
            )
        }
    }
}

fn render_config_table(
    strings: &I18nMap,
    rule_code: &str,
    rules_default_config: &Value,
    rules_schema: &Value,
) -> Option<String> {
    let entry = rules_default_config.pointer(&format!("/{rule_code}"))?.as_object()?;
    if entry.is_empty() {
        return None;
    }

    let option_h = t(strings, "rules_config_option", "Option");
    let type_h = t(strings, "rules_config_type", "Type");
    let default_h = t(strings, "rules_config_default", "Default");
    let mut out = format!("| {option_h} | {type_h} | {default_h} |\n");
    out.push_str("| :--- | :--- | :--- |\n");

    let mut keys: Vec<&String> = entry.keys().collect();
    keys.sort();
    for key in keys {
        let value = &entry[key];
        let type_label = schema_property(rules_schema, rules_schema, rule_code)
            .and_then(|rule| schema_property(rules_schema, rule, key))
            .and_then(schema_type_label)
            .unwrap_or_else(|| json_type_label(value).to_string());
        let value_label = json_value_label(value);
        out.push_str(&format!("| `{key}` | `{type_label}` | `{value_label}` |\n"));
    }
    out.push('\n');
    Some(out)
}

fn schema_property<'a>(root: &'a Value, schema: &'a Value, name: &str) -> Option<&'a Value> {
    if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
        return schema_property(root, root.pointer(reference.strip_prefix('#')?)?, name);
    }
    if let Some(property) = schema.get("properties").and_then(|properties| properties.get(name)) {
        return Some(property);
    }
    schema.get("allOf")?.as_array()?.iter().find_map(|part| schema_property(root, part, name))
}

fn schema_type_label(schema: &Value) -> Option<String> {
    let type_name = |kind: &str| if kind == "integer" { "number".to_string() } else { kind.to_string() };
    match schema.get("type")? {
        Value::String(kind) => Some(type_name(kind)),
        Value::Array(kinds) => {
            Some(kinds.iter().filter_map(Value::as_str).map(type_name).collect::<Vec<_>>().join(" or "))
        }
        _ => None,
    }
}

fn json_type_label(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn json_value_label(value: &Value) -> String {
    match value {
        Value::String(s) => format!("\"{}\"", s.to_lowercase()),
        Value::Array(_) | Value::Object(_) => serde_json::to_string(value).unwrap_or_else(|_| String::from("…")),
        _ => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_types_match_non_null_defaults() {
        let defaults = serde_json::to_value(RulesSettings::default()).unwrap();
        let schema = serde_json::to_value(schema_for!(RulesSettings)).unwrap();
        for (rule, options) in defaults.as_object().unwrap() {
            for (option, value) in options.as_object().unwrap() {
                if value.is_null() {
                    continue;
                }
                let rule_schema = schema_property(&schema, &schema, rule).unwrap();
                let option_schema = schema_property(&schema, rule_schema, option).unwrap();
                let kind = schema_type_label(option_schema).unwrap_or_else(|| json_type_label(value).to_string());
                assert_eq!(kind, json_type_label(value), "{rule}.{option}");
            }
        }
    }

    #[test]
    fn twig_rules_have_settings_and_matching_schema_types() {
        let page = twig_page().unwrap();
        assert_eq!(page.rules.len(), 25);
        for rule in &page.rules {
            let options = page.default_config[&rule.config_key].as_object().unwrap();
            let rule_schema = schema_property(&page.schema, &page.schema, &rule.config_key).unwrap();
            for (option, value) in options {
                let option_schema = schema_property(&page.schema, rule_schema, option).unwrap();
                let kind = schema_type_label(option_schema).unwrap_or_else(|| json_type_label(value).to_string());
                assert_eq!(kind, json_type_label(value), "{}.{option}", rule.config_key);
            }
        }
    }

    #[test]
    fn referenced_level_uses_default_value_type() {
        let schema = serde_json::to_value(schema_for!(RulesSettings)).unwrap();
        let rule = schema_property(&schema, &schema, "cyclomatic-complexity").unwrap();
        let level = schema_property(&schema, rule, "level").unwrap();
        assert!(schema_type_label(level).is_none());
        assert_eq!(
            json_type_label(&serde_json::to_value(RulesSettings::default()).unwrap()["cyclomatic-complexity"]["level"]),
            "string"
        );
    }

    #[test]
    fn nullable_threshold_types_come_from_schema() {
        let schema = serde_json::to_value(schema_for!(RulesSettings)).unwrap();
        for (rule, option) in [
            ("cyclomatic-complexity", "method-threshold"),
            ("excessive-parameter-list", "constructor-threshold"),
            ("excessive-nesting", "function-like-threshold"),
        ] {
            let rule_schema = schema_property(&schema, &schema, rule).unwrap();
            let option_schema = schema_property(&schema, rule_schema, option).unwrap();
            assert_eq!(schema_type_label(option_schema).as_deref(), Some("number or null"));
        }
    }
}
