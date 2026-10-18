use mago_analyzer::settings::strictness::AnalyzerStrictnessPreset;

#[test]
fn init_strictness_presets_select_the_expected_overly_wide_throw_setting() {
    assert_eq!(
        AnalyzerStrictnessPreset::Relaxed.settings().to_toml_assignment(),
        "find-overly-wide-throw-types = false"
    );
    assert_eq!(
        AnalyzerStrictnessPreset::default().settings().to_toml_assignment(),
        "find-overly-wide-throw-types = false"
    );
    assert_eq!(AnalyzerStrictnessPreset::Strict.settings().to_toml_assignment(), "find-overly-wide-throw-types = true");
    assert_eq!(
        AnalyzerStrictnessPreset::Maximum.settings().to_toml_assignment(),
        "find-overly-wide-throw-types = true"
    );
}
