use mago::init_presets::AnalyzerStrictnessPreset;

#[test]
fn init_strictness_presets_select_the_expected_overly_wide_throw_setting() {
    assert!(!AnalyzerStrictnessPreset::Relaxed.find_overly_wide_throw_types());
    assert!(!AnalyzerStrictnessPreset::default().find_overly_wide_throw_types());
    assert!(AnalyzerStrictnessPreset::Strict.find_overly_wide_throw_types());
    assert!(AnalyzerStrictnessPreset::Maximum.find_overly_wide_throw_types());
}
