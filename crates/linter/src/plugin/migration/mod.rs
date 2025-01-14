use crate::definition::PluginDefinition;
use crate::plugin::migration::rules::php80::str_contains::StrContainsRule;
use crate::plugin::migration::rules::php80::str_starts_with::StrStartsWithRule;
use crate::plugin::migration::rules::php81::explicit_octal_notation::ExplicitOctalNotationRule;
use crate::plugin::migration::rules::php82::readonly_class_promotion::ReadonlyClassPromotionRule;

use crate::plugin::Plugin;
use crate::rule::Rule;

pub mod rules;

#[derive(Debug)]
pub struct MigrationPlugin;

impl Plugin for MigrationPlugin {
    fn get_definition(&self) -> PluginDefinition {
        PluginDefinition {
            name: "Migration",
            description: "Provides rules that help migrate code to newer PHP versions.",
            enabled_by_default: true,
        }
    }

    fn get_rules(&self) -> Vec<Box<dyn Rule>> {
        vec![
            // PHP 8.0
            Box::new(StrStartsWithRule),
            Box::new(StrContainsRule),
            // PHP 8.1
            Box::new(ExplicitOctalNotationRule),
            // PHP 8.2
            Box::new(ReadonlyClassPromotionRule),
        ]
    }
}
