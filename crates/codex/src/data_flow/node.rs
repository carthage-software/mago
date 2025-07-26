use std::hash::Hash;
use std::hash::Hasher;

use serde::Deserialize;
use serde::Serialize;

use mago_interner::StringIdentifier;
use mago_interner::ThreadedInterner;
use mago_span::Position;
use mago_span::Span;

use crate::identifier::function_like::FunctionLikeIdentifier;
use crate::identifier::method::MethodIdentifier;
use crate::misc::VariableIdentifier;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub enum VariableSourceKind {
    Default,
    PrivateParameter,
    NonPrivateParameter,
    RefParameter,
    ClosureParameter,
    ClosureUse,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash, PartialOrd, Ord)]
pub enum DataFlowNodeId {
    String(String),
    LocalString(String, Span),
    ArrayAssignment(Span),
    ArrayItem(String, Span),
    Return(Span),
    ForInit(usize, usize),
    Composition(Span),
    Var(VariableIdentifier, Span),
    VarNarrowedTo(String, StringIdentifier, Position),
    Parameter(VariableIdentifier, Span),
    UnlabelledSink(Span),
    ReferenceTo(FunctionLikeIdentifier),
    CallTo(FunctionLikeIdentifier),
    SpecializedCallTo(FunctionLikeIdentifier, Position),
    FunctionLikeArg(FunctionLikeIdentifier, u8),
    SpecializedFunctionLikeArg(FunctionLikeIdentifier, u8, Position),
    Property(StringIdentifier, StringIdentifier),
    SpecializedProperty(StringIdentifier, StringIdentifier, Span),
    PropertyFetch(VariableIdentifier, StringIdentifier, Position),
    FunctionLikeReference(FunctionLikeIdentifier, u8),
    SpecializedFunctionLikeReference(FunctionLikeIdentifier, u8, Position),
    ThisBeforeMethod(MethodIdentifier),
    SpecializedThisBeforeMethod(MethodIdentifier, Position),
    ThisAfterMethod(MethodIdentifier),
    SpecializedThisAfterMethod(MethodIdentifier, Position),
    Symbol(StringIdentifier),
    ShapeFieldAccess(StringIdentifier, String),
    InstanceMethodCall(Span),
}

impl DataFlowNodeId {
    /// Checks if this node ID directly references the given variable identifier.
    pub fn references_variable(&self, var_id_to_find: &VariableIdentifier) -> bool {
        match self {
            DataFlowNodeId::Var(var_id, _) | DataFlowNodeId::Parameter(var_id, _) => var_id == var_id_to_find,
            _ => false,
        }
    }

    pub fn to_string(&self, interner: &ThreadedInterner) -> String {
        match self {
            DataFlowNodeId::String(str) => str.clone(),
            DataFlowNodeId::LocalString(str, span) => {
                format!("{}-{}:{}-{}", str, interner.lookup(&span.start.source.0), span.start.offset, span.end.offset,)
            }
            DataFlowNodeId::Parameter(var_id, span) => {
                format!(
                    "param-{}-{}:{}-{}",
                    interner.lookup(&var_id.0),
                    interner.lookup(&span.start.source.0),
                    span.start.offset,
                    span.end.offset,
                )
            }
            DataFlowNodeId::Var(var_id, span) => {
                format!(
                    "{}-{}:{}-{}",
                    interner.lookup(&var_id.0),
                    interner.lookup(&span.start.source.0),
                    span.start.offset,
                    span.end.offset,
                )
            }
            DataFlowNodeId::VarNarrowedTo(var_id, symbol, position) => {
                format!(
                    "{} narrowed to {}-{}:{}",
                    var_id,
                    interner.lookup(symbol),
                    interner.lookup(&position.source.0),
                    position.offset,
                )
            }
            DataFlowNodeId::ArrayAssignment(span) => {
                format!(
                    "array-assignment-{}:{}-{}",
                    interner.lookup(&span.start.source.0),
                    span.start.offset,
                    span.end.offset,
                )
            }
            DataFlowNodeId::ArrayItem(key_value, span) => {
                format!(
                    "array[{}]-{}:{}-{}",
                    key_value,
                    interner.lookup(&span.start.source.0),
                    span.start.offset,
                    span.end.offset,
                )
            }
            DataFlowNodeId::Return(span) => {
                format!("return-{}:{}-{}", interner.lookup(&span.start.source.0), span.start.offset, span.end.offset,)
            }
            DataFlowNodeId::CallTo(functionlike_id) => {
                format!("call to {}", functionlike_id.as_string(interner))
            }
            DataFlowNodeId::SpecializedCallTo(functionlike_id, position) => {
                format!(
                    "call to {}-{}:{}",
                    functionlike_id.as_string(interner),
                    interner.lookup(&position.source.0),
                    position.offset,
                )
            }
            DataFlowNodeId::Property(classlike_name, property_name) => {
                format!("{}::${}", interner.lookup(classlike_name), interner.lookup(property_name))
            }
            DataFlowNodeId::SpecializedProperty(classlike_name, property_name, span) => {
                format!(
                    "{}::${}-{}:{}-{}",
                    interner.lookup(classlike_name),
                    interner.lookup(property_name),
                    interner.lookup(&span.start.source.0),
                    span.start.offset,
                    span.end.offset,
                )
            }
            DataFlowNodeId::FunctionLikeReference(functionlike_id, arg) => {
                format!("ref {}#{}", functionlike_id.as_string(interner), (arg + 1))
            }
            DataFlowNodeId::SpecializedFunctionLikeReference(functionlike_id, arg, position) => {
                format!(
                    "ref {}#{}-{}:{}",
                    functionlike_id.as_string(interner),
                    (arg + 1),
                    interner.lookup(&position.source.0),
                    position.offset,
                )
            }
            DataFlowNodeId::FunctionLikeArg(functionlike_id, arg) => {
                format!("{}#{}", functionlike_id.as_string(interner), (arg + 1))
            }
            DataFlowNodeId::SpecializedFunctionLikeArg(functionlike_id, arg, position) => {
                format!(
                    "{}#{}-{}:{}",
                    functionlike_id.as_string(interner),
                    (arg + 1),
                    interner.lookup(&position.source.0),
                    position.offset,
                )
            }
            DataFlowNodeId::PropertyFetch(lhs_var_id, property_name, position) => {
                format!(
                    "{}->{}-{}:{}",
                    interner.lookup(&lhs_var_id.0),
                    interner.lookup(property_name),
                    interner.lookup(&position.source.0),
                    position.offset,
                )
            }
            DataFlowNodeId::ThisBeforeMethod(method_id) => {
                format!(
                    "$this in {} before {}",
                    interner.lookup(method_id.get_class_name()),
                    interner.lookup(method_id.get_method_name())
                )
            }
            DataFlowNodeId::SpecializedThisBeforeMethod(method_id, position) => {
                format!(
                    "$this in {} before {}-{}:{}",
                    interner.lookup(method_id.get_class_name()),
                    interner.lookup(method_id.get_method_name()),
                    interner.lookup(&position.source.0),
                    position.offset,
                )
            }
            DataFlowNodeId::ThisAfterMethod(method_id) => {
                format!(
                    "$this in {} after {}",
                    interner.lookup(method_id.get_class_name()),
                    interner.lookup(method_id.get_method_name())
                )
            }
            DataFlowNodeId::SpecializedThisAfterMethod(method_id, position) => {
                format!(
                    "$this in {} after {}-{}:{}",
                    interner.lookup(method_id.get_class_name()),
                    interner.lookup(method_id.get_method_name()),
                    interner.lookup(&position.source.0),
                    position.offset,
                )
            }
            DataFlowNodeId::Symbol(id) => interner.lookup(id).to_string(),
            DataFlowNodeId::ShapeFieldAccess(type_name, key) => {
                format!("{}[{}]", interner.lookup(type_name), key)
            }
            DataFlowNodeId::Composition(span) => {
                format!(
                    "composition-{}:{}-{}",
                    interner.lookup(&span.start.source.0),
                    span.start.offset,
                    span.end.offset,
                )
            }
            DataFlowNodeId::ReferenceTo(functionlike_id) => {
                format!("fnref-{}", functionlike_id.as_string(interner))
            }
            DataFlowNodeId::ForInit(start_offset, end_offset) => {
                format!("for-init-{start_offset}-{end_offset}")
            }
            DataFlowNodeId::UnlabelledSink(span) => {
                format!(
                    "unlabelled-sink-{}:{}-{}",
                    interner.lookup(&span.start.source.0),
                    span.start.offset,
                    span.end.offset,
                )
            }
            DataFlowNodeId::InstanceMethodCall(span) => {
                format!(
                    "instance-method-call-{}:{}-{}",
                    interner.lookup(&span.start.source.0),
                    span.start.offset,
                    span.end.offset,
                )
            }
        }
    }

    pub fn to_label(&self, interner: &ThreadedInterner) -> String {
        match self {
            DataFlowNodeId::String(str) | DataFlowNodeId::LocalString(str, ..) => str.clone(),
            DataFlowNodeId::Parameter(var_id, ..) | DataFlowNodeId::Var(var_id, ..) => {
                interner.lookup(&var_id.0).to_string()
            }
            DataFlowNodeId::VarNarrowedTo(var_id, symbol, ..) => {
                format!("{} narrowed to {}", var_id, interner.lookup(symbol),)
            }
            DataFlowNodeId::ArrayAssignment(..) => "array-assignment".to_string(),
            DataFlowNodeId::ArrayItem(key_value, ..) => {
                format!("array[{key_value}]")
            }
            DataFlowNodeId::Return(..) => "return".to_string(),
            DataFlowNodeId::CallTo(functionlike_id) | DataFlowNodeId::SpecializedCallTo(functionlike_id, ..) => {
                format!("call to {}", functionlike_id.as_string(interner))
            }
            DataFlowNodeId::Property(classlike_name, property_name)
            | DataFlowNodeId::SpecializedProperty(classlike_name, property_name, ..) => {
                format!("{}::${}", interner.lookup(classlike_name), interner.lookup(property_name))
            }

            DataFlowNodeId::FunctionLikeReference(functionlike_id, arg)
            | DataFlowNodeId::SpecializedFunctionLikeReference(functionlike_id, arg, ..) => {
                format!("ref {}#{}", functionlike_id.as_string(interner), (arg + 1))
            }

            DataFlowNodeId::FunctionLikeArg(functionlike_id, arg)
            | DataFlowNodeId::SpecializedFunctionLikeArg(functionlike_id, arg, ..) => {
                format!("{}#{}", functionlike_id.as_string(interner), (arg + 1))
            }

            DataFlowNodeId::PropertyFetch(lhs_var_id, property_name, ..) => {
                format!("{}->{}", interner.lookup(&lhs_var_id.0), interner.lookup(property_name),)
            }

            DataFlowNodeId::ThisBeforeMethod(method_id)
            | DataFlowNodeId::SpecializedThisBeforeMethod(method_id, ..) => {
                format!(
                    "$this in {} before {}",
                    interner.lookup(method_id.get_class_name()),
                    interner.lookup(method_id.get_method_name())
                )
            }

            DataFlowNodeId::ThisAfterMethod(method_id) | DataFlowNodeId::SpecializedThisAfterMethod(method_id, ..) => {
                format!(
                    "$this in {} after {}",
                    interner.lookup(method_id.get_class_name()),
                    interner.lookup(method_id.get_method_name())
                )
            }

            DataFlowNodeId::Symbol(id) => interner.lookup(id).to_string(),
            DataFlowNodeId::ShapeFieldAccess(type_name, key) => {
                format!("{}[{}]", interner.lookup(type_name), key)
            }
            DataFlowNodeId::Composition(..) => "composition".to_string(),
            DataFlowNodeId::ReferenceTo(functionlike_id) => {
                format!("fnref-{}", functionlike_id.as_string(interner))
            }
            DataFlowNodeId::ForInit(start_offset, end_offset) => {
                format!("for-init-{start_offset}-{end_offset}")
            }
            DataFlowNodeId::UnlabelledSink(..) => panic!(),
            DataFlowNodeId::InstanceMethodCall(..) => "instance method call".to_string(),
        }
    }

    pub fn specialize(&self, position: Position) -> DataFlowNodeId {
        match self {
            DataFlowNodeId::CallTo(id) => DataFlowNodeId::SpecializedCallTo(*id, position),
            DataFlowNodeId::FunctionLikeArg(functionlike_id, arg) => {
                DataFlowNodeId::SpecializedFunctionLikeArg(*functionlike_id, *arg, position)
            }
            DataFlowNodeId::FunctionLikeReference(functionlike_id, arg) => {
                DataFlowNodeId::SpecializedFunctionLikeReference(*functionlike_id, *arg, position)
            }
            DataFlowNodeId::ThisBeforeMethod(method_id) => {
                DataFlowNodeId::SpecializedThisBeforeMethod(*method_id, position)
            }
            DataFlowNodeId::ThisAfterMethod(method_id) => {
                DataFlowNodeId::SpecializedThisAfterMethod(*method_id, position)
            }
            _ => {
                unreachable!("Cannot specialize {:?}", self)
            }
        }
    }

    pub fn unspecialize(&self) -> (DataFlowNodeId, Position) {
        match self {
            DataFlowNodeId::SpecializedCallTo(id, position) => (DataFlowNodeId::CallTo(*id), *position),
            DataFlowNodeId::SpecializedFunctionLikeArg(functionlike_id, arg, position) => {
                (DataFlowNodeId::FunctionLikeArg(*functionlike_id, *arg), *position)
            }
            DataFlowNodeId::SpecializedFunctionLikeReference(functionlike_id, arg, position) => {
                (DataFlowNodeId::FunctionLikeReference(*functionlike_id, *arg), *position)
            }
            DataFlowNodeId::SpecializedThisBeforeMethod(method_id, position) => {
                (DataFlowNodeId::ThisBeforeMethod(*method_id), *position)
            }
            DataFlowNodeId::SpecializedThisAfterMethod(method_id, position) => {
                (DataFlowNodeId::ThisAfterMethod(*method_id), *position)
            }
            _ => {
                panic!()
            }
        }
    }

    pub fn span(&self) -> Option<Span> {
        match self {
            DataFlowNodeId::LocalString(_, span) => Some(*span),
            DataFlowNodeId::ArrayAssignment(span) => Some(*span),
            DataFlowNodeId::ArrayItem(_, span) => Some(*span),
            DataFlowNodeId::Return(span) => Some(*span),
            DataFlowNodeId::Parameter(_, span) => Some(*span),
            DataFlowNodeId::Var(_, span) => Some(*span),
            DataFlowNodeId::SpecializedProperty(_, _, span) => Some(*span),
            DataFlowNodeId::Composition(span) => Some(*span),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct DataFlowNode {
    pub id: DataFlowNodeId,
    pub kind: DataFlowNodeKind,
}

impl PartialEq for DataFlowNode {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub enum DataFlowNodeKind {
    Vertex { span: Option<Span>, is_specialized: bool },
    VariableUseSource { span: Span, kind: VariableSourceKind, pure: bool, has_parent_nodes: bool, from_loop_init: bool },
    VariableUseSink { span: Span },
    ForLoopInit { variable: VariableIdentifier },
    DataSource { span: Span, target_id: String },
}

impl Hash for DataFlowNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

impl DataFlowNode {
    pub fn get_for_method_argument(
        function_like_id: FunctionLikeIdentifier,
        argument_offset: usize,
        argument_span: Option<Span>,
        span: Option<Span>,
    ) -> Self {
        let mut id = DataFlowNodeId::FunctionLikeArg(function_like_id, argument_offset as u8);
        let mut is_specialized = false;

        if let Some(span) = span {
            id = DataFlowNodeId::SpecializedFunctionLikeArg(function_like_id, argument_offset as u8, span.start);
            is_specialized = true;
        }

        DataFlowNode { id, kind: DataFlowNodeKind::Vertex { span: argument_span, is_specialized } }
    }

    pub fn get_for_property(property_id: (StringIdentifier, StringIdentifier)) -> Self {
        DataFlowNode {
            id: DataFlowNodeId::Property(property_id.0, property_id.1),
            kind: DataFlowNodeKind::Vertex { span: None, is_specialized: false },
        }
    }

    pub fn get_for_localized_property(
        property_id: (StringIdentifier, StringIdentifier),
        assignment_span: Span,
    ) -> Self {
        DataFlowNode {
            id: DataFlowNodeId::SpecializedProperty(property_id.0, property_id.1, assignment_span),
            kind: DataFlowNodeKind::Vertex { span: Some(assignment_span), is_specialized: false },
        }
    }

    pub fn get_for_method_argument_reference(
        function_like_id: FunctionLikeIdentifier,
        argument_offset: usize,
        argument_span: Option<Span>,
        span: Option<Span>,
    ) -> Self {
        let mut id = DataFlowNodeId::FunctionLikeReference(function_like_id, argument_offset as u8);
        let mut is_specialized = false;

        if let Some(span) = span {
            id = DataFlowNodeId::SpecializedFunctionLikeReference(function_like_id, argument_offset as u8, span.start);
            is_specialized = true;
        }

        DataFlowNode { id, kind: DataFlowNodeKind::Vertex { span: argument_span, is_specialized } }
    }

    pub fn get_for_this_before_method(
        method_id: MethodIdentifier,
        method_span: Option<Span>,
        span: Option<Span>,
    ) -> Self {
        let mut id = DataFlowNodeId::ThisBeforeMethod(method_id);
        let mut is_specialized = false;

        if let Some(span) = span {
            id = DataFlowNodeId::SpecializedThisBeforeMethod(method_id, span.start);
            is_specialized = true;
        }

        DataFlowNode { id, kind: DataFlowNodeKind::Vertex { span: method_span, is_specialized } }
    }

    pub fn get_for_this_after_method(
        method_id: MethodIdentifier,
        method_span: Option<Span>,
        span: Option<Span>,
    ) -> Self {
        let mut id = DataFlowNodeId::ThisAfterMethod(method_id);
        let mut is_specialized = false;

        if let Some(span) = span {
            id = DataFlowNodeId::SpecializedThisAfterMethod(method_id, span.start);
            is_specialized = true;
        }

        DataFlowNode { id, kind: DataFlowNodeKind::Vertex { span: method_span, is_specialized } }
    }

    pub fn get_for_lvar(variable: VariableIdentifier, assignment_span: Span) -> Self {
        DataFlowNode {
            id: DataFlowNodeId::Var(variable, assignment_span),
            kind: DataFlowNodeKind::Vertex { span: Some(assignment_span), is_specialized: false },
        }
    }

    pub fn get_for_array_assignment(assignment_span: Span) -> Self {
        DataFlowNode {
            id: DataFlowNodeId::ArrayAssignment(assignment_span),
            kind: DataFlowNodeKind::Vertex { span: Some(assignment_span), is_specialized: false },
        }
    }

    pub fn get_for_return_expr(assignment_span: Span) -> Self {
        DataFlowNode {
            id: DataFlowNodeId::Return(assignment_span),
            kind: DataFlowNodeKind::Vertex { span: Some(assignment_span), is_specialized: false },
        }
    }

    pub fn get_for_array_item(key_value: String, assignment_span: Span) -> Self {
        DataFlowNode {
            id: DataFlowNodeId::ArrayItem(key_value, assignment_span),
            kind: DataFlowNodeKind::Vertex { span: Some(assignment_span), is_specialized: false },
        }
    }

    pub fn get_for_local_string(variable: String, assignment_span: Span) -> Self {
        DataFlowNode {
            id: DataFlowNodeId::LocalString(variable, assignment_span),
            kind: DataFlowNodeKind::Vertex { span: Some(assignment_span), is_specialized: false },
        }
    }

    pub fn get_for_instance_method_call(assignment_span: Span) -> Self {
        DataFlowNode {
            id: DataFlowNodeId::InstanceMethodCall(assignment_span),
            kind: DataFlowNodeKind::Vertex { span: Some(assignment_span), is_specialized: false },
        }
    }

    pub fn get_for_local_property_access(
        lhs_variable: VariableIdentifier,
        property_name: StringIdentifier,
        assignment_span: Span,
    ) -> Self {
        DataFlowNode {
            id: DataFlowNodeId::PropertyFetch(lhs_variable, property_name, assignment_span.start),
            kind: DataFlowNodeKind::Vertex { span: Some(assignment_span), is_specialized: false },
        }
    }

    pub fn get_for_narrowing(variable: String, narrowed_symbol: StringIdentifier, assignment_span: Span) -> Self {
        DataFlowNode {
            id: DataFlowNodeId::VarNarrowedTo(variable, narrowed_symbol, assignment_span.start),
            kind: DataFlowNodeKind::Vertex { span: Some(assignment_span), is_specialized: false },
        }
    }

    pub fn get_for_type(name: StringIdentifier, def_span: Span) -> Self {
        DataFlowNode {
            id: DataFlowNodeId::Symbol(name),
            kind: DataFlowNodeKind::Vertex { span: Some(def_span), is_specialized: false },
        }
    }

    pub fn get_for_call(function_like_id: FunctionLikeIdentifier, assignment_span: Span) -> Self {
        DataFlowNode {
            id: DataFlowNodeId::SpecializedCallTo(function_like_id, assignment_span.start),
            kind: DataFlowNodeKind::Vertex { span: Some(assignment_span), is_specialized: false },
        }
    }

    pub fn get_for_composition(assignment_span: Span) -> Self {
        DataFlowNode {
            id: DataFlowNodeId::Composition(assignment_span),
            kind: DataFlowNodeKind::Vertex { span: Some(assignment_span), is_specialized: false },
        }
    }

    pub fn get_for_unlabelled_sink(assignment_span: Span) -> Self {
        Self {
            id: DataFlowNodeId::UnlabelledSink(assignment_span),
            kind: DataFlowNodeKind::VariableUseSink { span: assignment_span },
        }
    }

    pub fn get_for_variable_sink(label: VariableIdentifier, assignment_span: Span) -> Self {
        Self {
            id: DataFlowNodeId::Var(label, assignment_span),
            kind: DataFlowNodeKind::VariableUseSink { span: assignment_span },
        }
    }

    pub fn get_for_variable_source(
        label: VariableIdentifier,
        assignment_span: Span,
        pure: bool,
        has_parent_nodes: bool,
        from_loop_init: bool,
    ) -> Self {
        Self {
            id: DataFlowNodeId::Var(label, assignment_span),
            kind: DataFlowNodeKind::VariableUseSource {
                span: assignment_span,
                kind: VariableSourceKind::Default,
                pure,
                has_parent_nodes,
                from_loop_init,
            },
        }
    }

    pub fn get_for_method_return(
        function_like_id: FunctionLikeIdentifier,
        span: Option<Span>,
        specialization_span: Option<Span>,
    ) -> Self {
        let mut id = DataFlowNodeId::CallTo(function_like_id);
        let mut is_specialized = false;

        if let Some(specialization_span) = specialization_span {
            id = DataFlowNodeId::SpecializedCallTo(function_like_id, specialization_span.start);
            is_specialized = true;
        }

        DataFlowNode { id, kind: DataFlowNodeKind::Vertex { span, is_specialized } }
    }

    pub fn get_for_method_reference(functionlike_id: FunctionLikeIdentifier, span: Option<Span>) -> Self {
        DataFlowNode {
            id: DataFlowNodeId::ReferenceTo(functionlike_id),
            kind: DataFlowNodeKind::Vertex { span, is_specialized: false },
        }
    }

    #[inline]
    pub fn get_span(&self) -> Option<Span> {
        match &self.kind {
            DataFlowNodeKind::Vertex { span, .. } => *span,
            DataFlowNodeKind::DataSource { span, .. } => Some(*span),
            DataFlowNodeKind::VariableUseSource { span, .. } => Some(*span),
            DataFlowNodeKind::VariableUseSink { span } => Some(*span),
            DataFlowNodeKind::ForLoopInit { .. } => None,
        }
    }
}
