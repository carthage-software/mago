#![allow(clippy::expect_used, clippy::missing_panics_doc)]

use clap::ColorChoice;
use mago_php_version::PHPVersion;

mod utils {
    use clap::ColorChoice;

    pub fn should_use_colors(color_choice: ColorChoice) -> bool {
        matches!(color_choice, ColorChoice::Always)
    }
}

#[path = "../src/config/analyzer.rs"]
mod analyzer_configuration;

use analyzer_configuration::AnalyzerConfiguration;

#[test]
fn deferred_callable_options_are_deserialized_and_forwarded() {
    let configuration: AnalyzerConfiguration = toml::from_str(
        r#"
forbid-checked-exceptions-in-callables = true
allowed-checked-exception-callables = ["\\TrustedDispatcher::Consume:0"]
"#,
    )
    .unwrap();

    let settings = configuration.to_settings(PHPVersion::PHP85, ColorChoice::Never, false);

    assert!(settings.forbid_checked_exceptions_in_callables);
    assert!(settings.allowed_checked_exception_callables.contains(&mago_word::word(b"trusteddispatcher::consume:0")));
}
