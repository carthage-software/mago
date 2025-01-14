use mago_interner::ThreadedInterner;
use mago_linter::definition::RuleUsageExample;
use mago_linter::rule::Rule;
use mago_linter::settings::RuleSettings;
use mago_linter::settings::Settings;
use mago_linter::Linter;
use mago_semantics::Semantics;
use mago_source::SourceCategory::UserDefined;
use mago_source::SourceManager;

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
    let source_manager = SourceManager::new(interner.clone());

    let mut rule_settings = RuleSettings::enabled();
    for (option, value) in usage_example.options.iter() {
        rule_settings.options.insert(option.to_string(), value.clone());
    }

    let source_name = format!("{}.php", definition.get_slug());
    let source_id = source_manager.insert_content(source_name, usage_example.snippet.to_string(), UserDefined);
    let source = source_manager.load(&source_id).unwrap();

    let semantics = Semantics::build(&interner, source);
    let source = source_manager.load(&source_id).unwrap();
    let reflection = mago_reflector::reflect(&interner, &source, &semantics.program, &semantics.names);

    let settings = Settings::new().with_rule(format!("test/{}", definition.get_slug()), rule_settings);

    let mut linter = Linter::new(settings, interner.clone(), reflection);

    linter.add_rule("test", rule);

    let issues = linter.lint(&Semantics::build(&interner, source));

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
