use bumpalo::Bump;

use mago_collector::Collector;
use mago_database::file::File;
use mago_reporting::IssueCollection;
use mago_syntax::ast::Program;

use crate::report::breach::BoundaryBreach;
use crate::report::flaw::StructuralFlaw;

pub mod breach;
pub mod flaw;

const COLLECTOR_CATEGORIES: &[&str] = &["guard"];

#[derive(Debug, Default)]
pub struct FortressReport {
    pub boundary_breaches: Vec<BoundaryBreach>,
    pub structural_flaws: Vec<StructuralFlaw>,
}

impl FortressReport {
    pub fn is_empty(&self) -> bool {
        self.boundary_breaches.is_empty() && self.structural_flaws.is_empty()
    }

    pub fn report_into_issues(self, arena: &Bump, source_file: &File, program: &Program) -> IssueCollection {
        let mut collector = Collector::new(arena, source_file, program, COLLECTOR_CATEGORIES);
        for boundary_breach in self.boundary_breaches {
            collector.report_with_code(boundary_breach.vector.error_code(), boundary_breach.into());
        }

        for structural_flaw in self.structural_flaws {
            collector.report_with_code(structural_flaw.kind.error_code(), structural_flaw.into());
        }

        collector.finish()
    }
}
