use mago_codex::data_flow::graph::GraphKind;
use mago_php_version::PHPVersion;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    pub version: PHPVersion,
    pub find_unused_expressions: bool,
    pub find_unused_definitions: bool,
    pub analyze_dead_code: bool,
    pub allow_include: bool,
    pub analyze_effects: bool,
    pub memoize_properties: bool,
    pub graph_kind: GraphKind,
    pub diff: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self::new(PHPVersion::LATEST)
    }
}

impl Settings {
    pub fn new(version: PHPVersion) -> Self {
        Self {
            version,
            find_unused_expressions: true,
            find_unused_definitions: true,
            analyze_dead_code: false,
            allow_include: true,
            memoize_properties: true,
            analyze_effects: true,
            graph_kind: GraphKind::FunctionBody,
            diff: false,
        }
    }
}
