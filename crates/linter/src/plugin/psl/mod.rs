use indoc::indoc;

use crate::definition::PluginDefinition;
use crate::plugin::Plugin;
use crate::plugin::psl::rules::array_functions::ArrayFunctionsRule;
use crate::plugin::psl::rules::data_structures::DataStructuresRule;
use crate::plugin::psl::rules::datetime::DateTimeRule;
use crate::plugin::psl::rules::math_functions::MathFunctionsRule;
use crate::plugin::psl::rules::output::OutputRule;
use crate::plugin::psl::rules::randomness_functions::RandomnessFunctionsRule;
use crate::plugin::psl::rules::regex_functions::RegexFunctionsRule;
use crate::plugin::psl::rules::sleep_functions::SleepFunctionsRule;
use crate::plugin::psl::rules::string_functions::StringFunctionsRule;
use crate::rule::Rule;

pub mod rules;

#[derive(Debug)]
pub struct PslPlugin;

impl Plugin for PslPlugin {
    fn get_definition(&self) -> PluginDefinition {
        PluginDefinition {
            name: "Psl",
            description: indoc! {"
                Enforces the consistent usage of the PHP Standard Library (Psl). This plugin
                helps you replace built-in PHP functions with their Psl equivalents, promoting type safety,
                improved error handling, and a more functional programming style.
            "},
            enabled_by_default: false,
        }
    }

    fn get_rules(&self) -> Vec<Box<dyn Rule>> {
        vec![
            Box::new(ArrayFunctionsRule),
            Box::new(DataStructuresRule),
            Box::new(DateTimeRule),
            Box::new(MathFunctionsRule),
            Box::new(OutputRule),
            Box::new(RandomnessFunctionsRule),
            Box::new(RegexFunctionsRule),
            Box::new(SleepFunctionsRule),
            Box::new(StringFunctionsRule),
        ]
    }
}
