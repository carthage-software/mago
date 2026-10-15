//! Build one file's semantic index from its CST and resolved names.

use foldhash::HashMap;
use foldhash::HashSet;
use mago_names::ResolvedNames;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::cst::*;
use mago_syntax::walker::Walker;
use mago_syntax::walker::walk_program;

use super::ImportBinding;
use super::MemberSelector;
use super::Occurrence;
use super::RuntimeVariables;
use super::ScopeKind;
use super::SelectorKind;
use super::SemanticIndex;
use super::SymbolKey;
use super::SymbolKind;
use super::callables::Callables;
use super::range;
use super::scope::Bindings;
use crate::Range;

#[derive(Default)]
struct Builder<'arena> {
    locals: Bindings<'arena>,
    occurrences: Vec<Occurrence>,
    names: HashMap<u32, (SymbolKind, bool)>,
    excluded: HashSet<u32>,
    imports: Vec<ImportBinding>,
    namespace_range: Option<Range>,
    member_selectors: Vec<MemberSelector>,
    callables: Callables,
    classes: Vec<u32>,
    runtime_variables: Vec<RuntimeVariables>,
}

pub(crate) fn build(program: &Program<'_>, resolved: &ResolvedNames<'_>, source: &[u8]) -> SemanticIndex {
    let mut builder = Builder::default();
    builder.locals.enter(Range::new(0, source.len() as u32), ScopeKind::File, false);
    walk_program(&Collector, program, &mut builder);

    let mut globals = HashMap::default();
    for (start, end, name, imported) in resolved.iter() {
        let (kind, declaration) = builder.names.get(&start).copied().unwrap_or((SymbolKind::ClassLike, false));
        let raw = &source[start as usize..end as usize];
        let first_end = raw.iter().position(|byte| *byte == b'\\').unwrap_or(raw.len());
        let import = builder.imports.iter().position(|binding| {
            binding.target_range.start == start
                || (imported
                    && !declaration
                    && binding.namespace_range.contains(start)
                    && (if first_end < raw.len() {
                        binding.kind == SymbolKind::ClassLike
                    } else {
                        binding.kind == kind
                    })
                    && same_name(binding.kind, &raw[..first_end], &binding.alias)
                    && name
                        .get(..binding.target.len())
                        .is_some_and(|prefix| same_name(binding.kind, prefix, &binding.target))
                    && (name.len() == binding.target.len() || name.get(binding.target.len()) == Some(&b'\\')))
        });
        let alias_range = import
            .filter(|id| builder.imports[*id].target_range.start != start)
            .map(|_| Range::new(start, start + first_end as u32));
        let mut name = name.to_vec();
        let insensitive_end = if kind == SymbolKind::Constant {
            name.iter().rposition(|byte| *byte == b'\\').unwrap_or(0)
        } else {
            name.len()
        };
        name[..insensitive_end].make_ascii_lowercase();
        let key = SymbolKey { kind, name: name.into_boxed_slice(), scope: None, binding: None };
        let symbol = *globals.entry(key.clone()).or_insert_with(|| {
            let id = builder.locals.symbols.len() as u32;
            builder.locals.symbols.push(key);
            id
        });
        let tail = source[start as usize..end as usize].iter().rposition(|byte| *byte == b'\\');
        builder.occurrences.push(Occurrence {
            range: Range::new(start, end),
            selection_range: Range::new(start + tail.map_or(0, |index| index as u32 + 1), end),
            symbol,
            declaration,
            imported,
            import: import.map(|id| id as u32),
            alias_range,
        });
    }
    builder.occurrences.sort_unstable_by_key(|entry| entry.range.start);
    builder.member_selectors.sort_unstable_by_key(|entry| entry.range.start);
    let (callable_references, callable_class_literals) = builder.callables.finish();
    SemanticIndex {
        symbols: builder.locals.symbols.into_boxed_slice(),
        occurrences: builder.occurrences.into_boxed_slice(),
        scopes: builder.locals.scopes.into_boxed_slice(),
        imports: builder.imports.into_boxed_slice(),
        member_selectors: builder.member_selectors.into_boxed_slice(),
        callable_references,
        callable_class_literals,
        runtime_variables: builder.runtime_variables.into_boxed_slice(),
    }
}

impl<'arena> Builder<'arena> {
    fn selector(&mut self, span: Span, receiver: Option<Span>, kind: SelectorKind, dynamic: bool) {
        self.member_selectors.push(MemberSelector {
            range: range(span),
            receiver: receiver.map(range),
            owner: self.classes.last().copied(),
            kind,
            dynamic,
        });
    }

    fn import(&mut self, item: &UseItem<'arena>, kind: SymbolKind, prefix: &[u8]) {
        self.classify(item.name.span(), kind, false);
        let name = item.name.value();
        let mut target = prefix.to_vec();
        if !target.is_empty() {
            target.push(b'\\');
        }
        target.extend_from_slice(name.strip_prefix(b"\\").unwrap_or(name));
        let alias = item
            .alias
            .as_ref()
            .map_or_else(|| name.rsplit(|byte| *byte == b'\\').next().unwrap_or(name), |alias| alias.identifier.value);
        self.imports.push(ImportBinding {
            target_range: range(item.name.span()),
            alias_range: item.alias.as_ref().map(|alias| range(alias.identifier.span)),
            alias: alias.into(),
            kind,
            namespace_range: self.namespace_range.unwrap_or(self.locals.scopes[0].range),
            target: target.into_boxed_slice(),
        });
    }

    fn classify(&mut self, span: Span, kind: SymbolKind, declaration: bool) {
        self.names.insert(span.start.offset, (kind, declaration));
    }

    fn variable(&mut self, name: &'arena [u8], range: Range, selection_range: Range) {
        if crate::variables::is_builtin_variable(name) {
            return;
        }
        let (symbol, declaration) = self.locals.occurrence(name, range.start);
        self.occurrences.push(Occurrence {
            range,
            selection_range,
            symbol,
            declaration,
            imported: false,
            import: None,
            alias_range: None,
        });
    }

    fn runtime(&mut self, span: Span, name: Option<&[u8]>) {
        self.runtime_variables.push(RuntimeVariables {
            range: range(span),
            scope: self.locals.scope(),
            name: name.map(Into::into),
        });
    }

    fn runtime_call(&mut self, call: &FunctionCall<'arena>, name: &[u8]) {
        let name = name.strip_prefix(b"\\").unwrap_or(name);
        if name.eq_ignore_ascii_case(b"extract") || name.eq_ignore_ascii_case(b"get_defined_vars") {
            self.runtime(call.span(), None);
        } else if name.eq_ignore_ascii_case(b"compact") {
            for argument in &call.argument_list.arguments {
                match argument {
                    Argument::Positional(argument) if argument.ellipsis.is_none() => {
                        self.compacted(call.span(), argument.value);
                    }
                    _ => self.runtime(call.span(), None),
                }
            }
        }
    }

    fn compacted(&mut self, call: Span, value: &Expression<'arena>) {
        let elements = match value.unparenthesized() {
            Expression::Literal(Literal::String(string)) => return self.runtime(call, string.value),
            Expression::Array(array) => &array.elements,
            Expression::LegacyArray(array) => &array.elements,
            _ => return self.runtime(call, None),
        };
        for element in elements {
            match element {
                ArrayElement::Value(entry) => self.compacted(call, entry.value),
                ArrayElement::KeyValue(entry) => self.compacted(call, entry.value),
                ArrayElement::Variadic(_) => self.runtime(call, None),
                ArrayElement::Missing(_) => {}
            }
        }
    }

    fn write(&mut self, expression: &Expression<'arena>) {
        let elements = match expression.unparenthesized() {
            Expression::Variable(Variable::Direct(variable)) => {
                self.locals.write(variable.span.start.offset);
                return;
            }
            Expression::UnaryPrefix(UnaryPrefix { operator: UnaryPrefixOperator::Reference(_), operand }) => {
                self.write(operand);
                return;
            }
            Expression::Array(array) => &array.elements,
            Expression::List(list) => &list.elements,
            _ => return,
        };
        for element in elements {
            match element {
                ArrayElement::Value(entry) => self.write(entry.value),
                ArrayElement::KeyValue(entry) => self.write(entry.value),
                ArrayElement::Variadic(_) | ArrayElement::Missing(_) => {}
            }
        }
    }
}

fn same_name(kind: SymbolKind, left: &[u8], right: &[u8]) -> bool {
    if kind == SymbolKind::Constant { left == right } else { left.eq_ignore_ascii_case(right) }
}

struct Collector;

impl<'ast, 'arena> Walker<'ast, 'arena, Builder<'arena>> for Collector {
    fn walk_in_array(&self, node: &'ast Array<'arena>, b: &mut Builder<'arena>) {
        b.callables.array(&node.elements, b.classes.last().copied());
    }
    fn walk_in_legacy_array(&self, node: &'ast LegacyArray<'arena>, b: &mut Builder<'arena>) {
        b.callables.array(&node.elements, b.classes.last().copied());
    }
    fn walk_in_literal_string(&self, node: &'ast LiteralString<'arena>, b: &mut Builder<'arena>) {
        b.callables.literal(node, b.classes.last().copied());
    }
    fn walk_in_function(&self, node: &'ast Function<'arena>, b: &mut Builder<'arena>) {
        b.classify(node.name.span, SymbolKind::Function, true);
        b.locals.enter(range(node.span()), ScopeKind::Function, false);
    }
    fn walk_out_function(&self, _: &'ast Function<'arena>, b: &mut Builder<'arena>) {
        b.locals.leave();
    }
    fn walk_in_method(&self, node: &'ast Method<'arena>, b: &mut Builder<'arena>) {
        b.locals.enter(range(node.span()), ScopeKind::Method, node.modifiers.contains_static());
    }
    fn walk_out_method(&self, _: &'ast Method<'arena>, b: &mut Builder<'arena>) {
        b.locals.leave();
    }
    fn walk_in_closure(&self, node: &'ast Closure<'arena>, b: &mut Builder<'arena>) {
        b.locals.enter(range(node.span()), ScopeKind::Closure, node.r#static.is_some());
    }
    fn walk_out_closure(&self, _: &'ast Closure<'arena>, b: &mut Builder<'arena>) {
        b.locals.leave();
    }
    fn walk_in_arrow_function(&self, node: &'ast ArrowFunction<'arena>, b: &mut Builder<'arena>) {
        b.locals.enter(range(node.span()), ScopeKind::ArrowFunction, node.r#static.is_some());
    }
    fn walk_out_arrow_function(&self, _: &'ast ArrowFunction<'arena>, b: &mut Builder<'arena>) {
        b.locals.leave();
    }
    fn walk_in_property_hook(&self, node: &'ast PropertyHook<'arena>, b: &mut Builder<'arena>) {
        b.locals.enter(range(node.span()), ScopeKind::PropertyHook, false);
        if node.parameter_list.is_none() && node.name.value.eq_ignore_ascii_case(b"set") {
            b.locals.implicit_parameter(b"value");
        }
    }
    fn walk_out_property_hook(&self, _: &'ast PropertyHook<'arena>, b: &mut Builder<'arena>) {
        b.locals.leave();
    }

    fn walk_in_function_like_parameter(&self, node: &'ast FunctionLikeParameter<'arena>, b: &mut Builder<'arena>) {
        b.locals.parameter(&node.variable.name[1..], node.variable.span.start.offset, node.is_promoted_property());
    }
    fn walk_in_closure_use_clause_variable(
        &self,
        node: &'ast ClosureUseClauseVariable<'arena>,
        b: &mut Builder<'arena>,
    ) {
        b.locals.capture(&node.variable.name[1..]);
    }
    fn walk_in_direct_variable(&self, node: &'ast DirectVariable<'arena>, b: &mut Builder<'arena>) {
        if !b.excluded.contains(&node.span.start.offset) {
            b.variable(&node.name[1..], range(node.span), Range::new(node.span.start.offset + 1, node.span.end.offset));
        }
    }
    fn walk_in_indirect_variable(&self, node: &'ast IndirectVariable<'arena>, b: &mut Builder<'arena>) {
        let span = node.span();
        if b.excluded.contains(&span.start.offset) {
            return;
        }
        let label = match node.expression {
            Expression::ArrayAccess(access) => access.array,
            expression => expression,
        };
        match (label, node.expression) {
            (Expression::Identifier(Identifier::Local(label)), _) => {
                b.variable(label.value, range(label.span), range(label.span));
            }
            (_, Expression::Literal(Literal::String(string))) => b.runtime(span, string.value),
            _ => b.runtime(span, None),
        }
    }
    fn walk_in_eval_construct(&self, node: &'ast EvalConstruct<'arena>, b: &mut Builder<'arena>) {
        b.runtime(node.span(), None);
    }
    fn walk_in_nested_variable(&self, node: &'ast NestedVariable<'arena>, b: &mut Builder<'arena>) {
        let span = node.span();
        if !b.excluded.contains(&span.start.offset) {
            b.runtime(span, None);
        }
    }
    fn walk_in_assignment(&self, node: &'ast Assignment<'arena>, b: &mut Builder<'arena>) {
        b.write(node.lhs);
    }
    fn walk_in_foreach_value_target(&self, node: &'ast ForeachValueTarget<'arena>, b: &mut Builder<'arena>) {
        b.write(node.value);
    }
    fn walk_in_foreach_key_value_target(&self, node: &'ast ForeachKeyValueTarget<'arena>, b: &mut Builder<'arena>) {
        b.write(node.key);
        b.write(node.value);
    }
    fn walk_in_try_catch_clause(&self, node: &'ast TryCatchClause<'arena>, b: &mut Builder<'arena>) {
        if let Some(variable) = &node.variable {
            b.locals.write(variable.span.start.offset);
        }
    }
    fn walk_in_static_item(&self, node: &'ast StaticItem<'arena>, b: &mut Builder<'arena>) {
        b.locals.write(node.variable().span.start.offset);
    }
    fn walk_in_global(&self, node: &'ast Global<'arena>, b: &mut Builder<'arena>) {
        for variable in &node.variables {
            if let Variable::Direct(variable) = variable {
                b.locals.global(&variable.name[1..]);
            }
        }
    }
    fn walk_in_property_item(&self, node: &'ast PropertyItem<'arena>, b: &mut Builder<'arena>) {
        b.excluded.insert(node.variable().span.start.offset);
    }
    fn walk_in_static_property_access(&self, node: &'ast StaticPropertyAccess<'arena>, b: &mut Builder<'arena>) {
        b.selector(
            node.property.span(),
            Some(node.class.span()),
            SelectorKind::Property,
            !matches!(node.property, Variable::Direct(_)),
        );
        b.excluded.insert(node.property.span().start.offset);
    }

    fn walk_in_class(&self, node: &'ast Class<'arena>, b: &mut Builder<'arena>) {
        b.classes.push(node.span().start.offset);
        b.classify(node.name.span, SymbolKind::ClassLike, true);
    }
    fn walk_in_interface(&self, node: &'ast Interface<'arena>, b: &mut Builder<'arena>) {
        b.classes.push(node.span().start.offset);
        b.classify(node.name.span, SymbolKind::ClassLike, true);
    }
    fn walk_in_trait(&self, node: &'ast Trait<'arena>, b: &mut Builder<'arena>) {
        b.classes.push(node.span().start.offset);
        b.classify(node.name.span, SymbolKind::ClassLike, true);
    }
    fn walk_in_enum(&self, node: &'ast Enum<'arena>, b: &mut Builder<'arena>) {
        b.classes.push(node.span().start.offset);
        b.classify(node.name.span, SymbolKind::ClassLike, true);
    }
    fn walk_in_namespace(&self, node: &'ast Namespace<'arena>, b: &mut Builder<'arena>) {
        b.namespace_range = Some(range(node.span()));
        if let Some(name) = &node.name {
            b.classify(name.span(), SymbolKind::Namespace, true);
        }
    }
    fn walk_out_namespace(&self, _: &'ast Namespace<'arena>, b: &mut Builder<'arena>) {
        b.namespace_range = None;
    }
    fn walk_out_class(&self, _: &'ast Class<'arena>, b: &mut Builder<'arena>) {
        b.classes.pop();
    }
    fn walk_out_interface(&self, _: &'ast Interface<'arena>, b: &mut Builder<'arena>) {
        b.classes.pop();
    }
    fn walk_out_trait(&self, _: &'ast Trait<'arena>, b: &mut Builder<'arena>) {
        b.classes.pop();
    }
    fn walk_out_enum(&self, _: &'ast Enum<'arena>, b: &mut Builder<'arena>) {
        b.classes.pop();
    }
    fn walk_in_anonymous_class(&self, node: &'ast AnonymousClass<'arena>, b: &mut Builder<'arena>) {
        b.classes.push(node.span().start.offset);
    }
    fn walk_out_anonymous_class(&self, _: &'ast AnonymousClass<'arena>, b: &mut Builder<'arena>) {
        b.classes.pop();
    }
    fn walk_in_method_call(&self, node: &'ast MethodCall<'arena>, b: &mut Builder<'arena>) {
        b.selector(node.method.span(), Some(node.object.span()), SelectorKind::Method, !node.method.is_identifier());
    }
    fn walk_in_null_safe_method_call(&self, node: &'ast NullSafeMethodCall<'arena>, b: &mut Builder<'arena>) {
        b.selector(node.method.span(), Some(node.object.span()), SelectorKind::Method, !node.method.is_identifier());
    }
    fn walk_in_static_method_call(&self, node: &'ast StaticMethodCall<'arena>, b: &mut Builder<'arena>) {
        b.selector(node.method.span(), Some(node.class.span()), SelectorKind::Method, !node.method.is_identifier());
    }
    fn walk_in_method_partial_application(
        &self,
        node: &'ast MethodPartialApplication<'arena>,
        b: &mut Builder<'arena>,
    ) {
        b.selector(node.method.span(), Some(node.object.span()), SelectorKind::Method, !node.method.is_identifier());
    }
    fn walk_in_static_method_partial_application(
        &self,
        node: &'ast StaticMethodPartialApplication<'arena>,
        b: &mut Builder<'arena>,
    ) {
        b.selector(node.method.span(), Some(node.class.span()), SelectorKind::Method, !node.method.is_identifier());
    }
    fn walk_in_property_access(&self, node: &'ast PropertyAccess<'arena>, b: &mut Builder<'arena>) {
        b.selector(
            node.property.span(),
            Some(node.object.span()),
            SelectorKind::Property,
            !node.property.is_identifier(),
        );
    }
    fn walk_in_null_safe_property_access(&self, node: &'ast NullSafePropertyAccess<'arena>, b: &mut Builder<'arena>) {
        b.selector(
            node.property.span(),
            Some(node.object.span()),
            SelectorKind::Property,
            !node.property.is_identifier(),
        );
    }
    fn walk_in_class_constant_access(&self, node: &'ast ClassConstantAccess<'arena>, b: &mut Builder<'arena>) {
        b.selector(
            node.constant.span(),
            Some(node.class.span()),
            SelectorKind::Constant,
            !node.constant.is_identifier(),
        );
    }
    fn walk_in_trait_use_alias_adaptation(&self, node: &'ast TraitUseAliasAdaptation<'arena>, b: &mut Builder<'arena>) {
        let (name, receiver) = match &node.method_reference {
            TraitUseMethodReference::Identifier(name) => (name.span, None),
            TraitUseMethodReference::Absolute(reference) => {
                (reference.method_name.span, Some(reference.trait_name.span()))
            }
        };
        b.selector(name, receiver, SelectorKind::TraitAdaptation, false);
        if let Some(alias) = node.alias {
            b.selector(alias.span, receiver, SelectorKind::TraitAlias, false);
        }
    }
    fn walk_in_trait_use_precedence_adaptation(
        &self,
        node: &'ast TraitUsePrecedenceAdaptation<'arena>,
        b: &mut Builder<'arena>,
    ) {
        b.selector(
            node.method_reference.method_name.span,
            Some(node.method_reference.trait_name.span()),
            SelectorKind::TraitAdaptation,
            false,
        );
    }
    fn walk_in_constant(&self, node: &'ast Constant<'arena>, b: &mut Builder<'arena>) {
        for item in &node.items {
            b.classify(item.name.span, SymbolKind::Constant, true);
        }
    }
    fn walk_in_constant_access(&self, node: &'ast ConstantAccess<'arena>, b: &mut Builder<'arena>) {
        b.classify(node.name.span(), SymbolKind::Constant, false);
    }
    fn walk_in_function_call(&self, node: &'ast FunctionCall<'arena>, b: &mut Builder<'arena>) {
        if let Expression::Identifier(name) = node.function {
            b.classify(name.span(), SymbolKind::Function, false);
            if !name.is_qualified() {
                b.runtime_call(node, name.value());
            }
        }
    }
    fn walk_in_function_partial_application(
        &self,
        node: &'ast FunctionPartialApplication<'arena>,
        b: &mut Builder<'arena>,
    ) {
        if let Expression::Identifier(name) = node.function {
            b.classify(name.span(), SymbolKind::Function, false);
        }
    }
    fn walk_in_use(&self, node: &'ast Use<'arena>, b: &mut Builder<'arena>) {
        let kind = |typed: Option<&UseType<'_>>| match typed {
            Some(UseType::Function(_)) => SymbolKind::Function,
            Some(UseType::Const(_)) => SymbolKind::Constant,
            None => SymbolKind::ClassLike,
        };
        match &node.items {
            UseItems::Sequence(items) => {
                for item in &items.items {
                    b.import(item, kind(None), b"");
                }
            }
            UseItems::TypedSequence(items) => {
                for item in &items.items {
                    b.import(item, kind(Some(&items.r#type)), b"");
                }
            }
            UseItems::TypedList(items) => {
                b.classify(items.namespace.span(), SymbolKind::Namespace, false);
                for item in &items.items {
                    b.import(
                        item,
                        kind(Some(&items.r#type)),
                        items.namespace.value().strip_prefix(b"\\").unwrap_or(items.namespace.value()),
                    );
                }
            }
            UseItems::MixedList(items) => {
                b.classify(items.namespace.span(), SymbolKind::Namespace, false);
                for item in &items.items {
                    b.import(
                        &item.item,
                        kind(item.r#type.as_ref()),
                        items.namespace.value().strip_prefix(b"\\").unwrap_or(items.namespace.value()),
                    );
                }
            }
        }
    }
}
