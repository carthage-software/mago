use std::rc::Rc;

use ahash::HashMap;

use mago_codex::assertion::Assertion;
use mago_codex::data_flow::graph::DataFlowGraph;
use mago_codex::reference::SymbolReferences;
use mago_codex::ttype::template::TemplateBound;
use mago_codex::ttype::union::TUnion;
use mago_span::HasSpan;

#[derive(Debug, Clone)]
pub struct AnalysisArtifacts {
    pub expression_types: HashMap<(usize, usize), Rc<TUnion>>,
    pub if_true_assertions: HashMap<(usize, usize), HashMap<String, Vec<Assertion>>>,
    pub if_false_assertions: HashMap<(usize, usize), HashMap<String, Vec<Assertion>>>,
    pub data_flow_graph: DataFlowGraph,
    pub inferred_return_types: Vec<TUnion>,
    pub closure_spans: Vec<(usize, usize)>,
    pub symbol_references: SymbolReferences,
    pub type_variable_bounds: HashMap<String, (Vec<TemplateBound>, Vec<TemplateBound>)>,
}

impl AnalysisArtifacts {
    pub(crate) fn new(data_flow_graph: DataFlowGraph) -> Self {
        Self {
            expression_types: HashMap::default(),
            data_flow_graph,
            inferred_return_types: Vec::new(),
            closure_spans: vec![],
            if_true_assertions: HashMap::default(),
            if_false_assertions: HashMap::default(),
            symbol_references: SymbolReferences::new(),
            type_variable_bounds: HashMap::default(),
        }
    }

    /// Set the type of expression `expression` to `t`.
    #[inline]
    pub fn set_expression_type<T: HasSpan>(&mut self, expression: &T, t: TUnion) {
        self.expression_types.insert(get_expression_range(expression), Rc::new(t));
    }

    /// Get the type of expression `expression`.
    #[inline]
    pub fn get_expression_type<T: HasSpan>(&self, expression: &T) -> Option<&TUnion> {
        let t = self.expression_types.get(&get_expression_range(expression))?;

        Some(&**t)
    }

    /// Set the type of expression `expression` to `t`.
    #[inline]
    pub fn set_rc_expression_type<T: HasSpan>(&mut self, expression: &T, t: Rc<TUnion>) {
        self.expression_types.insert(get_expression_range(expression), t);
    }

    /// Get the type of expression `expression`.
    #[inline]
    pub fn get_rc_expression_type<T: HasSpan>(&self, expression: &T) -> Option<&Rc<TUnion>> {
        self.expression_types.get(&get_expression_range(expression))
    }
}

#[inline]
pub fn get_expression_range<T: HasSpan>(expression: &T) -> (usize, usize) {
    let span = expression.span();

    (span.start.offset, span.end.offset)
}
