use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::category::Category;
use crate::context::LintContext;
use crate::integration::Integration;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct PreferAnonymousMigrationRule {
    meta: &'static RuleMeta,
    cfg: PreferAnonymousMigrationConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct PreferAnonymousMigrationConfig {
    pub level: Level,
}

impl Default for PreferAnonymousMigrationConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for PreferAnonymousMigrationConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for PreferAnonymousMigrationRule {
    type Config = PreferAnonymousMigrationConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Prefer Anonymous Migration",
            code: "prefer-anonymous-migration",
            description: indoc! {"
                Prefer using anonymous classes for Laravel migrations instead of named classes.
                Anonymous classes are more concise and reduce namespace pollution,
                making them the recommended approach for migrations.
            "},
            good_example: indoc! {r#"
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
            bad_example: indoc! {r#"
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
            category: Category::BestPractices,
            requirements: RuleRequirements::Integration(Integration::Laravel),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::AnonymousClass];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::AnonymousClass(class) = node else {
            return;
        };

        let Some(extends) = class.extends.as_ref() else {
            return;
        };

        let mut is_migration = false;
        for extended_type in extends.types.iter() {
            let name = ctx.lookup_name(extended_type);

            if name.eq_ignore_ascii_case("Illuminate\\Database\\Migrations\\Migration") {
                is_migration = true;
                break;
            }
        }

        if !is_migration {
            return;
        }

        ctx.collector.report(
            Issue::new(self.cfg.level(), "Use anonymous classes for migrations instead of named classes.")
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(class.span()).with_message("This migration class should be anonymous"),
                )
                .with_note("Anonymous classes are the recommended approach for Laravel migrations.")
                .with_help("Refactor the migration to use an anonymous class by removing the class name."),
        );
    }
}
