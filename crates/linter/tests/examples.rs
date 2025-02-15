use mago_interner::ThreadedInterner;
use mago_linter::definition::RuleUsageExample;
use mago_linter::rule::Rule;
use mago_linter::settings::RuleSettings;
use mago_linter::settings::Settings;
use mago_linter::Linter;
use mago_php_version::PHPVersion;
use mago_project::module::Module;
use mago_source::Source;

pub mod plugins;

#[macro_export]
macro_rules! rule_test {
    ($name:ident, $rule:expr) => {
        #[test]
        fn $name() {
            use mago_linter::rule::Rule;

            let rule = $rule;
            for usage_example in rule.get_definition().examples {
                $crate::test_rule_usage_example(Box::new($rule), &usage_example);
            }
        }
    };
}

pub fn test_rule_usage_example(rule: Box<dyn Rule>, usage_example: &RuleUsageExample) {
    let definition = rule.get_definition();

    let interner = ThreadedInterner::new();

    let mut rule_settings = RuleSettings::enabled();
    for (option, value) in usage_example.options.iter() {
        rule_settings.options.insert(option.to_string(), value.clone());
    }

    let source_name = format!("{}.php", definition.get_slug());
    let source = Source::standalone(&interner, &source_name, usage_example.snippet);

    let mut php_version = PHPVersion::PHP84;
    if let Some(version) = rule.get_definition().maximum_supported_php_version {
        php_version = PHPVersion::from_version_id(version.to_version_id() - 1);
    }
    if let Some(version) = rule.get_definition().minimum_supported_php_version {
        php_version = version;
    }

    let settings = Settings::new(php_version).with_rule(format!("test/{}", definition.get_slug()), rule_settings);

    let mut module = Module::build(&interner, php_version, source, Default::default());
    let mut linter = Linter::new(settings, interner.clone(), module.reflection.take().unwrap_or_default());

    linter.add_rule("test", rule);

    let issues = linter.lint(&module);

    if usage_example.valid {
        assert!(
            issues.is_empty(),
            "Rule `{}` example `{}` should not have issues, but got: {:?}",
            definition.get_slug(),
            usage_example.description,
            issues
        );
    } else {
        assert!(
            !issues.is_empty(),
            "Rule `{}` example `{}` should have issues, but got none.",
            definition.get_slug(),
            usage_example.description
        );
    }
}
