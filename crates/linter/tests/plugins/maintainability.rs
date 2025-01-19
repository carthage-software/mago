use mago_linter::plugin::maintainability::rules::cyclomatic_complexity::CyclomaticComplexityRule;
use mago_linter::plugin::maintainability::rules::excessive_parameter_list::ExcessiveParameterListRule;
use mago_linter::plugin::maintainability::rules::halstead::HalsteadRule;
use mago_linter::plugin::maintainability::rules::kan_defect::KanDefectRule;
use mago_linter::plugin::maintainability::rules::too_many_enum_cases::TooManyEnumCasesRule;
use mago_linter::plugin::maintainability::rules::too_many_methods::TooManyMethodsRule;
use mago_linter::plugin::maintainability::rules::too_many_properties::TooManyPropertiesRule;

use crate::rule_test;

rule_test!(test_cyclomatic_complexity, CyclomaticComplexityRule);
rule_test!(test_excessive_parameter_list, ExcessiveParameterListRule);
rule_test!(test_halstead, HalsteadRule);
rule_test!(test_kan_defect, KanDefectRule);
rule_test!(test_too_many_enum_cases, TooManyEnumCasesRule);
rule_test!(test_too_many_methods, TooManyMethodsRule);
rule_test!(test_too_many_properties, TooManyPropertiesRule);
