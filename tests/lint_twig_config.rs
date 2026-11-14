mod common_twig_lint;

use common_twig_lint::*;

#[test]
fn settings_inherit_and_schema_lists_all_twig_fields() {
    let directory = workspace("[twig.linter]\nversion='3.30'\n[twig.linter.rules.no-dump]\nlevel='error'\n");
    let root = directory.path();
    std::fs::rename(root.join("mago.toml"), root.join("base.toml")).unwrap();
    std::fs::write(
        root.join("mago.toml"),
        "extends='base.toml'\n[twig.linter.rules.no-dump]\nexclude=['src/skip.twig']\n",
    )
    .unwrap();
    let output = run(root, &["config", "--show", "twig.linter"]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let config: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(config["enabled"], true);
    assert_eq!(config["version"], "3.30");
    assert_eq!(config["rules"]["no-dump"]["level"], "Error");
    assert_eq!(config["rules"]["no-dump"]["exclude"][0], "src/skip.twig");
    let output = stdin(root, "src/new.twig", b"{{ dump(user) }}");
    assert_eq!(output.status.code(), Some(1));
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["issues"][0]["level"], "Error");
    assert!(
        serde_json::from_slice::<serde_json::Value>(&stdin(root, "src/skip.twig", b"{{ dump(user) }}").stdout).unwrap()
            ["issues"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    let output = run(root, &["config", "--schema", "--show", "twig.linter"]);
    assert!(output.status.success());
    let schema: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    for field in ["enabled", "version", "excludes", "rules", "baseline", "baseline-variant", "minimum-fail-level"] {
        assert!(schema["properties"][field].is_object(), "Missing field {field}");
    }
}

#[test]
fn nested_container_display_schema_and_lazy_defaults_match() {
    let directory = workspace("");
    let root = directory.path();
    let output = run(root, &["config", "--show", "twig"]);
    assert!(output.status.success());
    let twig: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(twig["linter"]["version"], "3.30");
    assert_eq!(twig["linter"]["enabled"], true);
    let output = run(root, &["config"]);
    let config: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(config["twig"], twig);
    assert!(config.get("twig-linter").is_none());

    let output = run(root, &["config", "--schema", "--show", "twig"]);
    assert!(output.status.success());
    let schema: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let property = &schema["properties"]["linter"];
    let reference = property["$ref"]
        .as_str()
        .or_else(|| property["anyOf"].as_array()?.iter().find_map(|branch| branch["$ref"].as_str()))
        .unwrap();
    let linter = schema.pointer(reference.strip_prefix('#').unwrap()).unwrap();
    for field in ["enabled", "version", "excludes", "rules", "baseline", "baseline-variant", "minimum-fail-level"] {
        assert!(linter["properties"][field].is_object(), "Missing field {field}");
    }
    assert_eq!(schema["additionalProperties"], false);

    std::fs::write(root.join("mago.toml"), "[source]\npaths=['src']\n").unwrap();
    let output = run(root, &["config"]);
    let config: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(config.get("twig").is_none());
    let output = run(root, &["config", "--show", "twig"]);
    assert_eq!(serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap(), serde_json::json!({}));
    let output = run(root, &["config", "--show", "twig.linter"]);
    let linter: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(linter["version"], "3.0");
    assert_eq!(linter["enabled"], true);
}

#[test]
fn old_flat_table_and_unknown_nested_fields_are_errors() {
    for configuration in ["[twig-linter]\nenabled=true\n", "[twig]\nlinetr={}\n", "[twig.linter]\nunknown=true\n"] {
        let directory = workspace(configuration);
        assert!(!run(directory.path(), &["twig:lint"]).status.success(), "{configuration}");
    }
    let directory = workspace("");
    assert!(!run(directory.path(), &["config", "--show", "twig-linter"]).status.success());
}

#[test]
fn disabled_rules_and_severities_follow_config_but_only_can_enable_a_rule() {
    let directory = workspace("[twig.linter.rules.named-argument-separator]\nenabled=false\nlevel='error'\n");
    assert!(issues(directory.path(), &["src/template.twig"]).as_array().unwrap().is_empty());
    let result = issues(directory.path(), &["src/template.twig", "--only", "twig-named-argument-separator"]);
    assert_eq!(result.as_array().unwrap().len(), 2);
    assert!(result.as_array().unwrap().iter().all(|issue| issue["level"] == "Error"));
    let output = run(directory.path(), &["twig:lint", "--only", "twig-named-argument-separator"]);
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn unknown_settings_fail_and_old_versions_never_get_a_new_separator_fix() {
    for configuration in [
        "[twig.linter]\nverison='3.10'\n",
        "[twig.linter.rules.disallowed-functions]\nnmes=['source']\n",
        "[twig.linter]\nunknown=true\n",
        "[twig.linter.rules.no-dump]\nunknown=true\n",
        "[twig.linter.rules.no-such-rule]\nenabled=true\n",
        "[twig.linter]\nversion='4.0'\n",
    ] {
        let directory = workspace(configuration);
        assert!(!run(directory.path(), &["twig:lint"]).status.success());
    }
    let directory = workspace("[twig.linter]\nversion='3.11'\n");
    let before = std::fs::read(directory.path().join("src/template.twig")).unwrap();
    assert!(
        run(directory.path(), &["twig:lint", "src/template.twig", "--only", "twig-named-argument-separator", "--fix"])
            .status
            .success()
    );
    assert_eq!(std::fs::read(directory.path().join("src/template.twig")).unwrap(), before);
}

#[test]
fn default_minimum_version_never_adds_a_modern_separator() {
    let directory = workspace("");
    let root = directory.path();
    std::fs::write(root.join("mago.toml"), "[source]\npaths=['src']\n").unwrap();
    let output = run(root, &["config", "--show", "twig.linter"]);
    let config: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(config["version"], "3.0");
    let before = std::fs::read(root.join("src/template.twig")).unwrap();
    assert!(
        run(root, &["twig:lint", "src/template.twig", "--only", "twig-named-argument-separator", "--fix"])
            .status
            .success()
    );
    assert_eq!(std::fs::read(root.join("src/template.twig")).unwrap(), before);
}

#[test]
fn only_selects_rules_without_skipping_twig_syntax_or_parsing_php() {
    let directory = workspace("");
    let root = directory.path();
    std::fs::write(root.join("src/main.php"), b"<?php function main( {").unwrap();
    std::fs::write(root.join("src/template.twig"), b"{{ ( }}").unwrap();
    for only in ["strict-types", "twig-no-dump"] {
        let result = issues(root, &["--only", only]);
        let issues = result.as_array().unwrap();
        assert!(issues.iter().any(|issue| issue["code"] == "twig-syntax-error"));
        assert!(issues.iter().all(|issue| issue["annotations"][0]["span"]["file_id"]["name"] == "src/template.twig"));
    }
}

#[test]
fn disabled_rule_expectations_follow_php_behavior_except_for_only_runs() {
    let directory = workspace("[twig.linter.rules.no-dump]\nenabled=false\n");
    let root = directory.path();
    std::fs::write(root.join("src/template.twig"), "{# @mago-expect lint:twig-no-dump #}\n{{ dump(value) }}").unwrap();
    let result = issues(root, &["src/template.twig"]);
    assert_eq!(result.as_array().unwrap().len(), 1);
    assert_eq!(result[0]["code"], "unfulfilled-expect");
    assert!(issues(root, &["src/template.twig", "--only", "twig-no-dump"]).as_array().unwrap().is_empty());
    assert!(issues(root, &["src/template.twig", "--only", "twig-single-quotes"]).as_array().unwrap().is_empty());
}

#[test]
fn configured_name_bans_are_separate_from_unknown_extension_names() {
    let directory =
        workspace("[twig.linter.rules.disallowed-functions]\nenabled=true\nlevel='error'\nnames=['source']\n");
    std::fs::write(directory.path().join("src/template.twig"), "{{ custom() }}{{ source('secret') }}").unwrap();
    let result = issues(directory.path(), &["src/template.twig"]);
    assert_eq!(result.as_array().unwrap().len(), 1);
    assert_eq!(result[0]["code"], "twig-disallowed-functions");
    assert_eq!(result[0]["level"], "Error");
}

#[test]
fn semantics_only_ignores_lint_policies_and_literal_php_text() {
    let directory = workspace("[twig.linter.rules.no-dump]\nlevel='error'\n");
    let source = b"<?php broken(\n{{ dump(user) }}";
    std::fs::write(directory.path().join("src/template.twig"), source).unwrap();
    assert!(issues(directory.path(), &["src/template.twig", "--semantics"]).as_array().unwrap().is_empty());
    assert_eq!(issues(directory.path(), &["src/template.twig"]).as_array().unwrap().len(), 1);
    std::fs::write(directory.path().join("src/template.twig"), "{% if %}").unwrap();
    assert_eq!(issues(directory.path(), &["src/template.twig", "--semantics"])[0]["code"], "twig-syntax-error");
}

#[test]
fn counted_pragmas_keep_nested_scopes_and_only_skips_inactive_codes() {
    let directory = workspace("");
    let source = include_bytes!("fixtures/lint_twig/pragmas.twig");
    std::fs::write(directory.path().join("src/template.twig"), source).unwrap();
    let result = issues(directory.path(), &["src/template.twig", "--only", "twig-no-dump"]);
    assert!(result.as_array().unwrap().is_empty(), "{result}");
    let result = issues(directory.path(), &["src/template.twig", "--only", "twig-single-quotes"]);
    assert!(result.as_array().unwrap().is_empty(), "{result}");
}

#[test]
fn pragma_gaps_and_inline_boundaries_do_not_suppress_other_tags() {
    let directory = workspace("");
    let root = directory.path();
    for (source, code, count) in [
        (include_bytes!("../crates/twig-linter/tests/fixtures/orphan_pragmas.twig").as_slice(), "twig-no-dump", 2),
        (
            include_bytes!("../crates/twig-linter/tests/fixtures/inline_pragmas.twig").as_slice(),
            "twig-single-quotes",
            3,
        ),
    ] {
        std::fs::write(root.join("src/template.twig"), source).unwrap();
        let result = issues(root, &["src/template.twig", "--only", code]);
        assert_eq!(result.as_array().unwrap().len(), 4, "{result}");
        assert_eq!(result.as_array().unwrap().iter().filter(|issue| issue["code"] == code).count(), count);
        assert!(issues(root, &["src/template.twig", "--semantics"]).as_array().unwrap().is_empty());
    }
}

#[test]
fn raw_policy_never_offers_a_fix_at_any_safety_level() {
    let directory = workspace("[twig.linter.rules.no-raw]\nenabled=true\n");
    let root = directory.path();
    let source = include_bytes!("../crates/twig-linter/tests/fixtures/raw_boundaries.twig");
    for flags in [vec![], vec!["--potentially-unsafe"], vec!["--unsafe"]] {
        std::fs::write(root.join("src/template.twig"), source).unwrap();
        let mut args = vec!["twig:lint", "src/template.twig", "--only", "twig-no-raw", "--fix"];
        args.extend(flags);
        assert!(run(root, &args).status.success());
        assert_eq!(std::fs::read(root.join("src/template.twig")).unwrap(), source);
        assert!(issues(root, &["src/template.twig", "--semantics"]).as_array().unwrap().is_empty());
    }
    assert!(
        !run(
            root,
            &["twig:lint", "src/template.twig", "--only", "twig-no-raw", "--fix", "--unsafe", "--fail-on-remaining"]
        )
        .status
        .success()
    );
}

#[test]
fn legacy_raw_files_keep_all_source_bytes_at_every_fix_level() {
    let directory = workspace(
        "[twig.linter.rules.single-quotes]\nenabled=true\n[twig.linter.rules.end-block-name]\nenabled=true\n",
    );
    let root = directory.path();
    let source = include_bytes!("../crates/twig-linter/tests/fixtures/legacy_raw.twig");
    for only in ["twig-named-argument-separator,twig-single-quotes,twig-end-block-name", "strict-types"] {
        for flags in [vec![], vec!["--potentially-unsafe"], vec!["--unsafe"]] {
            std::fs::write(root.join("src/template.twig"), source).unwrap();
            let mut arguments = vec!["twig:lint", "src/template.twig", "--only", only, "--fix"];
            arguments.extend(flags);
            assert!(run(root, &arguments).status.success());
            assert_eq!(std::fs::read(root.join("src/template.twig")).unwrap(), source);
        }
    }
    assert!(run(root, &["fix", "src/template.twig"]).status.success());
    assert_eq!(std::fs::read(root.join("src/template.twig")).unwrap(), source);
    assert!(issues(root, &["src/template.twig", "--semantics"]).as_array().unwrap().is_empty());
}

#[test]
fn semantics_rejects_invalid_assignment_targets() {
    let directory = workspace("");
    let root = directory.path();
    for source in
        include_str!("../crates/twig-syntax/tests/fixtures/expressions/invalid_assignment_targets.twig").lines()
    {
        std::fs::write(root.join("src/template.twig"), source).unwrap();
        let result = issues(root, &["src/template.twig", "--semantics"]);
        let expected = if source == "{{ {\"a#{b}\": name} = {} }}" { 3 } else { 1 };
        assert_eq!(result.as_array().unwrap().len(), expected, "{source}: {result}");
        assert!(result.as_array().unwrap().iter().all(|issue| issue["code"] == "twig-syntax-error"), "{source}");
        if expected == 3 {
            for issue in result.as_array().unwrap() {
                assert_eq!(issue["annotations"][0]["span"]["start"]["offset"], 4);
                assert_eq!(issue["annotations"][0]["span"]["end"]["offset"], 5);
            }
        }
    }
}

#[test]
fn twig_command_keeps_php_text_out_of_php_checks_with_explicit_source_extensions() {
    let directory = workspace("");
    let root = directory.path();
    std::fs::write(root.join("mago.toml"), "[source]\npaths=['src']\nextensions=['php','twig']\n").unwrap();
    let source = b"<?php broken(\n<p>{{ value }}</p>\n";
    std::fs::write(root.join("src/template.twig"), source).unwrap();
    let output = run(root, &["twig:lint", "src/template.twig", "--semantics", "--reporting-format", "json"]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(result["issues"].as_array().unwrap().is_empty());
    assert!(run(root, &["twig:lint", "src/template.twig", "--fix"]).status.success());
    assert_eq!(std::fs::read(root.join("src/template.twig")).unwrap(), source);
}
