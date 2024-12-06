use fennec_ast::Program;
use fennec_interner::ThreadedInterner;
use fennec_parser::error::ParseError;
use fennec_source::error::SourceError;
use fennec_source::SourceIdentifier;
use fennec_source::SourceManager;

#[derive(Debug)]
pub struct AstService {
    interner: ThreadedInterner,
    source_manager: SourceManager,
}

impl AstService {
    pub fn new(interner: ThreadedInterner, source_manager: SourceManager) -> Self {
        Self { interner, source_manager }
    }

    ///  Parse the given bytes into an AST.
    pub async fn parse(&self, source: SourceIdentifier) -> Result<(Program, Option<ParseError>), SourceError> {
        let source = self.source_manager.load(source)?;

        Ok(fennec_parser::parse_source(&self.interner, &source))
    }
}
