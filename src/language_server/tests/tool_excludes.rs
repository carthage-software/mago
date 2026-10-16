//! Per-tool `excludes` withhold one subsystem's output and keep the file navigable.

use foldhash::HashMap;
use serde_json::Value;
use serde_json::json;

use super::harness::Harness;

const CONFIG: &str = r#"
[analyzer]
excludes = ["Unanalyzed.php", "Quiet.php", "broken-*.php"]

[linter]
excludes = ["Unlinted.php", "Quiet*.php", "broken-quiet.php"]

[formatter]
excludes = ["unformatted.php"]
"#;

const MESSY: &str = "<?php\n\ndeclare(strict_types=1);\n\nfunction messy(){return 1;}\n";

fn source(class: &str) -> String {
    format!(
        "<?php\n\ndeclare(strict_types=1);\n\nfinal class {class}\n{{\n    public function run(): int\n    {{\n        return (integer) '1';\n    }}\n\n    private function unused(): void {{}}\n}}\n"
    )
}

async fn codes(h: &mut Harness) -> HashMap<String, Vec<String>> {
    let mut latest = HashMap::default();
    for publication in h.diagnostics().await {
        let mut codes: Vec<_> = publication["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .map(|diagnostic| diagnostic["code"].as_str().unwrap_or_default().to_owned())
            .collect();
        codes.sort_unstable();
        latest.insert(publication["uri"].as_str().unwrap().to_owned(), codes);
    }
    latest
}

async fn fixes(h: &mut Harness, name: &str) -> Vec<String> {
    let actions = h
        .request(
            "textDocument/codeAction",
            json!({
                "textDocument": { "uri": h.url(name) },
                "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 13, "character": 0 } },
                "context": { "diagnostics": [], "only": ["quickfix"] }
            }),
        )
        .await;
    let mut codes: Vec<_> = actions
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .filter_map(|action| action["diagnostics"][0]["code"].as_str().map(str::to_owned))
        .collect();
    codes.sort_unstable();
    codes
}

#[tokio::test]
async fn tool_excludes_withhold_only_their_own_diagnostics_fixes_and_edits() {
    let files = [
        ("Normal.php", source("Normal")),
        ("Unanalyzed.php", source("Unanalyzed")),
        ("Unlinted.php", source("Unlinted")),
        ("Quiet.php", source("Quiet")),
        ("broken-unanalyzed.php", "<?php\nfunction unanalyzed( {}\n".to_owned()),
        ("broken-quiet.php", "<?php\nfunction quiet( {}\n".to_owned()),
        ("messy.php", MESSY.to_owned()),
        ("unformatted.php", MESSY.replace("messy", "unformatted")),
        ("mago.toml", CONFIG.to_owned()),
    ];
    let files: Vec<_> = files.iter().map(|(name, contents)| (*name, contents.as_str())).collect();
    let mut h = Harness::start(&files).await;
    let published = codes(&mut h).await;
    let codes = |name: &str| published.get(&h.url(name)).cloned().unwrap_or_default();

    let both = ["deprecated-cast", "unused-method"];
    assert_eq!(codes("Normal.php"), both);
    assert_eq!(codes("Unanalyzed.php"), ["deprecated-cast"]);
    assert_eq!(codes("Unlinted.php"), ["unused-method"]);
    assert_eq!(codes("Quiet.php"), Vec::<String>::new());
    assert!(
        codes("broken-unanalyzed.php").iter().any(|code| code == "parse" || code == "syntax"),
        "{:?}",
        codes("broken-unanalyzed.php")
    );
    assert_eq!(codes("broken-quiet.php"), Vec::<String>::new());

    assert_eq!(fixes(&mut h, "Normal.php").await, both);
    assert_eq!(fixes(&mut h, "Unanalyzed.php").await, ["deprecated-cast"]);
    assert_eq!(fixes(&mut h, "Unlinted.php").await, ["unused-method"]);
    assert_eq!(fixes(&mut h, "Quiet.php").await, Vec::<String>::new());

    let hover = h.at("textDocument/hover", "Quiet.php", 11, 22).await;
    assert!(hover["contents"]["value"].as_str().is_some_and(|value| value.contains("function unused")), "{hover}");
    let definition = h.at("textDocument/definition", "Quiet.php", 11, 22).await;
    assert_eq!(definition["uri"], h.url("Quiet.php"), "{definition}");

    for (name, formatted) in [("messy.php", true), ("unformatted.php", false)] {
        let edits: Value = h
            .request(
                "textDocument/formatting",
                json!({ "textDocument": { "uri": h.url(name) }, "options": { "tabSize": 4, "insertSpaces": true } }),
            )
            .await;
        assert_eq!(!edits.is_null(), formatted, "{name}: {edits}");
    }
}

#[tokio::test]
async fn formatter_excludes_prune_directories_in_every_source_tier() {
    let config = r#"
[source]
paths = ["src"]
includes = ["vendor"]
patches = ["patches"]

[source.glob]
literal-separator = true

[formatter]
excludes = ["vendor/generated/*", "patches/generated/*"]
"#;
    let files = [
        ("mago.toml", config),
        ("src/messy.php", MESSY),
        ("vendor/generated/deep/Service.php", MESSY),
        ("vendor/other/Kept.php", MESSY),
        ("patches/generated/deep/Patch.php", MESSY),
    ];
    let mut h = Harness::start(&files).await;
    let range = json!({ "start": { "line": 0, "character": 0 }, "end": { "line": 5, "character": 0 } });
    for (name, formatted) in [
        ("src/messy.php", true),
        ("vendor/other/Kept.php", true),
        ("vendor/generated/deep/Service.php", false),
        ("patches/generated/deep/Patch.php", false),
    ] {
        let options = json!({ "tabSize": 4, "insertSpaces": true });
        let document = h
            .request("textDocument/formatting", json!({ "textDocument": { "uri": h.url(name) }, "options": options }))
            .await;
        assert_eq!(!document.is_null(), formatted, "{name}: {document}");
        let ranged = h
            .request(
                "textDocument/rangeFormatting",
                json!({ "textDocument": { "uri": h.url(name) }, "range": range, "options": options }),
            )
            .await;
        assert_eq!(!ranged.is_null(), formatted, "{name}: {ranged}");
    }
}
