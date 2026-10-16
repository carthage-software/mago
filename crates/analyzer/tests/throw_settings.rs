use mago_analyzer::settings::Settings;
use mago_php_version::PHPVersion;

#[test]
fn overly_wide_throw_setting_defaults_off_and_is_independent_of_unhandled_throw_check() {
    let mut settings = Settings::new(PHPVersion::PHP85);

    assert!(!settings.find_overly_wide_throw_types);

    settings.check_throws = true;
    assert!(!settings.find_overly_wide_throw_types);

    settings.find_overly_wide_throw_types = true;
    assert!(settings.find_overly_wide_throw_types);
    assert!(settings.check_throws);
}
