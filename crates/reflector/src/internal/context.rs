use mago_interner::ThreadedInterner;
use mago_names::Names;
use mago_source::Source;

#[derive(Debug)]
pub struct Context<'a> {
    pub interner: &'a ThreadedInterner,
    pub source: &'a Source,
    pub names: &'a Names,
}

impl<'a> Context<'a> {
    pub fn new(interner: &'a ThreadedInterner, source: &'a Source, names: &'a Names) -> Self {
        Self { interner, source, names }
    }
}
