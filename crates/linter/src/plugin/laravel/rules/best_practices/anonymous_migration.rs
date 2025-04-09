use indoc::indoc;

use mago_reporting::*;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct AnonymousMigrationRule;

impl Rule for AnonymousMigrationRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Anonymous Migration", Level::Warning)
            .with_description(indoc! {"
                Prefer using anonymous classes for Laravel migrations instead of named classes.

                Anonymous classes are more concise and reduce namespace pollution,
                making them the recommended approach for migrations.
            "})
            .with_example(RuleUsageExample::valid(
                "Using an anonymous migration class.",
                indoc! {r#"
                    <?php

                    use Illuminate\Database\Migrations\Migration;
                    use Illuminate\Database\Schema\Blueprint;
                    use Illuminate\Support\Facades\Schema;

                    return new class extends Migration {
                        public function up(): void {
                            Schema::create('flights', function (Blueprint $table) {
                                $table->id();
                                $table->string('name');
                                $table->string('airline');
                                $table->timestamps();
                            });
                        }

                        public function down(): void {
                            Schema::drop('flights');
                        }
                    };
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Using a named migration class.",
                indoc! {r#"
                    <?php

                    use Illuminate\Database\Migrations\Migration;
                    use Illuminate\Database\Schema\Blueprint;
                    use Illuminate\Support\Facades\Schema;

                    class MyMigration extends Migration {
                        public function up(): void {
                            Schema::create('flights', function (Blueprint $table) {
                                $table->id();
                                $table->string('name');
                                $table->string('airline');
                                $table->timestamps();
                            });
                        }

                        public function down(): void {
                            Schema::drop('flights');
                        }
                    }

                    return new MyMigration();
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::Class(class) = node else {
            return LintDirective::default();
        };

        let Some(extends) = class.extends.as_ref() else {
            return LintDirective::default();
        };

        let mut is_migration = false;
        for extended_type in extends.types.iter() {
            let name = context.lookup_name(&extended_type);

            if name.eq_ignore_ascii_case("Illuminate\\Database\\Migrations\\Migration") {
                is_migration = true;
                break;
            }
        }

        if !is_migration {
            return LintDirective::default();
        }

        context.report(
            Issue::new(context.level(), "Use anonymous classes for migrations instead of named classes.")
                .with_annotation(
                    Annotation::primary(class.span()).with_message("This migration class should be anonymous."),
                )
                .with_note("Anonymous classes are the recommended approach for Laravel migrations.")
                .with_help("Refactor the migration to use an anonymous class by removing the class name."),
        );

        LintDirective::Prune
    }
}
