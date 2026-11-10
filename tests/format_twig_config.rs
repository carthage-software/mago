mod common_twig_format;

use common_twig_format::*;

#[test]
fn settings_have_their_own_namespace_and_schema() {
    let directory = workspace(include_str!("fixtures/format_twig/settings.toml"));
    let root = directory.path();
    let php = json(root, &["config", "--show", "formatter"]);
    assert_eq!(php["print-width"], 20);
    assert!(php.get("twig").is_none());
    let twig = json(root, &["config", "--show", "twig"]);
    assert_eq!(twig["formatter"]["tab-width"], 2);
    let formatter = json(root, &["config", "--show", "twig.formatter"]);
    assert_eq!(formatter["print-width"], 80);
    assert_eq!(formatter["tab-width"], 2);
    assert_eq!(formatter["enabled"], true);
    let all = json(root, &["config"]);
    assert_eq!(all["twig"]["formatter"]["tab-width"], 2);
    assert!(all["formatter"].get("twig").is_none());

    let schema = json(root, &["config", "--schema", "--show", "twig.formatter"]);
    for name in ["enabled", "excludes", "print-width", "tab-width", "use-tabs", "end-of-line"] {
        assert!(schema["properties"][name].is_object(), "Missing setting: {name}");
    }
    let schema = json(root, &["config", "--schema", "--show", "twig"]);
    assert!(schema["properties"]["formatter"].is_object());
    let php_schema = json(root, &["config", "--schema", "--show", "formatter"]);
    assert!(php_schema["properties"].get("twig").is_none());

    let output = stdin(root, Some("src/template.html.twig"), BEFORE);
    assert!(output.status.success());
    assert_eq!(output.stdout, include_bytes!("fixtures/format_twig/two_spaces.twig"));
}

#[test]
fn missing_twig_settings_stay_lazy_and_have_tool_defaults() {
    let directory = workspace(CONFIG);
    let root = directory.path();
    assert!(json(root, &["config"]).get("twig").is_none());
    assert_eq!(json(root, &["config", "--show", "twig"]), serde_json::json!({}));
    let defaults = json(root, &["config", "--show", "twig.formatter", "--default"]);
    assert_eq!(defaults["enabled"], true);
    assert_eq!(defaults["tab-width"], 4);
    assert_eq!(defaults["print-width"], 120);
}

#[test]
fn rejects_old_and_unknown_settings_without_aliases() {
    for config in [
        include_str!("fixtures/format_twig/unknown_setting.toml"),
        include_str!("fixtures/format_twig/legacy.toml"),
        "[twig]\nunknown=true\n",
        "[twig-formatter]\nenabled=true\n",
    ] {
        let directory = workspace(config);
        let output = run(directory.path(), &["twig:fmt"]);
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("unknown"));
    }
    let directory = workspace(CONFIG);
    for name in ["twig-formatter", "formatter.twig"] {
        assert!(!run(directory.path(), &["config", "--show", name]).status.success());
    }
}

#[test]
fn inherits_nested_settings_through_configuration_layers() {
    let directory = workspace(include_str!("fixtures/format_twig/inherits.toml"));
    let root = directory.path();
    std::fs::write(root.join("base.toml"), include_str!("fixtures/format_twig/settings.toml")).unwrap();
    let formatter = json(root, &["config", "--show", "twig.formatter"]);
    assert_eq!(formatter["print-width"], 60);
    assert_eq!(formatter["tab-width"], 2);
    assert_eq!(json(root, &["config", "--show", "formatter"])["print-width"], 20);
    assert_eq!(stdin(root, None, BEFORE).stdout, include_bytes!("fixtures/format_twig/two_spaces.twig"));
}

#[test]
fn reads_nested_json_configuration() {
    let directory = workspace(CONFIG);
    let root = directory.path();
    let path = root.join("mago.json");
    std::fs::write(&path, include_bytes!("fixtures/format_twig/settings.json")).unwrap();
    let output = run(root, &["--config", path.to_str().unwrap(), "config", "--show", "twig.formatter"]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let config: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(config["tab-width"], 2);
}
