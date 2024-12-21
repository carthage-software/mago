use ahash::HashSet;

use mago_ast::*;
use mago_interner::StringIdentifier;
use mago_interner::ThreadedInterner;
use mago_names::Names;
use mago_reflection::identifier::ClassLikeName;
use mago_reflection::identifier::FunctionLikeName;
use mago_reflection::r#type::kind::*;
use mago_reflection::CodebaseReflection;
use mago_source::Source;
use mago_span::HasPosition;
use mago_span::HasSpan;
use mago_trinary::Trinary;

use crate::constant::ConstantTypeResolver;
use crate::internal::*;

/// A basic type resolver designed to initialize types at the beginning of type checking.
/// This resolver is intentionally simple and acts as a "bootstrap" to get type information
/// that can later be narrowed down for more precise type inference.
///
/// While this type resolver is limited in detail, it provides enough information for initial
/// analysis and bootstrapping. However, it requires a subsequent pass to refine types.
///
/// ### Capabilities
///
/// This resolver can completely evaluate constant expressions in most cases and produce a type
/// with a known value, such as `Value::Float(16.5)` for expressions like `((12 / 2) + 5) * 1.5`.
///
/// ### Improved Type Resolution with Codebase Context
///
/// If a codebase is available, this resolver can leverage it to retrieve types of functions,
/// methods, and constants, making it slightly more powerful in providing initial type information.
pub struct TypeResolver<'i, 'c> {
    interner: &'i ThreadedInterner,
    source: &'c Source,
    names: &'c Names,
    codebase: Option<&'c CodebaseReflection>,
    constant_resolver: ConstantTypeResolver<'i, 'c>,
}

impl<'i, 'c> TypeResolver<'i, 'c> {
    pub fn new(
        interner: &'i ThreadedInterner,
        source: &'c Source,
        names: &'c Names,
        codebase: Option<&'c CodebaseReflection>,
    ) -> Self {
        Self {
            interner,
            source,
            names,
            codebase,
            constant_resolver: ConstantTypeResolver::new(interner, names, codebase),
        }
    }

    pub fn resolve(&self, expression: &Expression) -> TypeKind {
        match expression {
            Expression::Parenthesized(parenthesized) => self.resolve(&parenthesized.expression),
            Expression::Binary(operation) => get_binary_operation_kind(self.interner, operation, |e| self.resolve(e)),
            Expression::UnaryPrefix(operation) => {
                get_unary_prefix_operation_kind(self.interner, operation, |e| self.resolve(e))
            }
            Expression::UnaryPostfix(operation) => get_unary_postfix_operation_kind(operation, |e| self.resolve(e)),
            Expression::Literal(literal) => get_literal_kind(self.interner, literal),
            Expression::CompositeString(composite_string) => {
                get_composite_string_kind(composite_string, |e| self.resolve(e))
            }
            Expression::AssignmentOperation(assignment_operation) => self.resolve(&assignment_operation.rhs),
            Expression::Conditional(conditional) => get_conditional_kind(conditional, |e| self.resolve(e)),
            Expression::Array(array) => get_array_kind(&array.elements, |e| self.resolve(e)),
            Expression::LegacyArray(legacy_array) => get_array_kind(&legacy_array.elements, |e| self.resolve(e)),
            Expression::ArrayAccess(array_access) => get_array_index_kind(self.resolve(&array_access.array)),
            Expression::AnonymousClass(anonymous_class) => anonymous_object_kind(anonymous_class.span()),
            Expression::Closure(closure) => {
                if let Some(codebase) = self.codebase {
                    if let Some(function) = codebase.get_function_like(FunctionLikeName::ArrowFunction(closure.span()))
                    {
                        return TypeKind::from(function);
                    }
                }

                // could be better..
                any_closure_kind()
            }
            Expression::ArrowFunction(arrow_function) => {
                if let Some(codebase) = self.codebase {
                    if let Some(function) =
                        codebase.get_function_like(FunctionLikeName::ArrowFunction(arrow_function.span()))
                    {
                        return TypeKind::from(function);
                    }
                }

                // could be better..
                any_closure_kind()
            }
            Expression::ConstantAccess(access) => self.constant_resolver.resolve(&access.name),
            Expression::Match(match_expression) => {
                let mut kinds = HashSet::default();
                for arm in match_expression.arms.iter() {
                    match &arm {
                        MatchArm::Expression(match_expression_arm) => {
                            kinds.insert(self.resolve(&match_expression_arm.expression));
                        }
                        MatchArm::Default(match_default_arm) => {
                            kinds.insert(self.resolve(&match_default_arm.expression));
                        }
                    }
                }

                if kinds.is_empty() {
                    never_kind()
                } else {
                    union_kind(kinds.into_iter().collect())
                }
            }
            Expression::Construct(construct) => match construct.as_ref() {
                Construct::Isset(_) => bool_kind(),
                Construct::Empty(_) => bool_kind(),
                Construct::Eval(_) => mixed_kind(false),
                Construct::Include(_) => mixed_kind(false),
                Construct::IncludeOnce(_) => mixed_kind(false),
                Construct::Require(_) => mixed_kind(false),
                Construct::RequireOnce(_) => mixed_kind(false),
                Construct::Print(_) => value_integer_kind(1),
                Construct::Exit(_) => never_kind(),
                Construct::Die(_) => never_kind(),
            },
            Expression::Throw(_) => never_kind(),
            Expression::Clone(clone) => {
                let object_kind = self.resolve(&clone.object);
                if object_kind.is_object() {
                    return object_kind;
                }

                any_object_kind()
            }
            Expression::Call(call) => match call {
                Call::Function(function_call) => {
                    if let Some(codebase) = self.codebase {
                        if let Expression::Identifier(identifier) = function_call.function.as_ref() {
                            let (full_name, short_name) = resolve_name(self.interner, identifier.value());

                            if let Some(function) = codebase.get_function(&full_name) {
                                return function.return_type_reflection.as_ref().map_or_else(
                                    || mixed_kind(false),
                                    |return_type| return_type.type_reflection.kind.clone(),
                                );
                            }

                            if let Some(function) = codebase.get_function(&short_name) {
                                return function.return_type_reflection.as_ref().map_or_else(
                                    || mixed_kind(false),
                                    |return_type| return_type.type_reflection.kind.clone(),
                                );
                            }
                        }
                    }

                    mixed_kind(false)
                }
                Call::Method(method_call) => {
                    let object_kind = self.resolve(&method_call.object);

                    if let TypeKind::Object(object_kind) = object_kind {
                        let ClassLikeMemberSelector::Identifier(method) = &method_call.method else {
                            return mixed_kind(false);
                        };

                        if let Some(codebase) = self.codebase {
                            let class_like_reflection = match &object_kind {
                                ObjectTypeKind::NamedObject { name, .. } => codebase.get_named_class_like(name),
                                ObjectTypeKind::AnonymousObject { span } => {
                                    codebase.get_class_like(ClassLikeName::AnonymousClass(*span))
                                }
                                ObjectTypeKind::EnumCase { enum_name, .. } => codebase.get_enum(enum_name),
                                _ => {
                                    return mixed_kind(false);
                                }
                            };

                            if let Some(class_reflection) = class_like_reflection {
                                if let Some(method) = class_reflection.get_method(&method.value) {
                                    return method.return_type_reflection.as_ref().map_or_else(
                                        || mixed_kind(false),
                                        |return_type| return_type.type_reflection.kind.clone(),
                                    );
                                }
                            }
                        }
                    }

                    mixed_kind(false)
                }
                Call::NullSafeMethod(null_safe_method_call) => {
                    let object_kind = self.resolve(&null_safe_method_call.object);

                    if let TypeKind::Object(object_kind) = object_kind {
                        let ClassLikeMemberSelector::Identifier(method) = &null_safe_method_call.method else {
                            return mixed_kind(false);
                        };

                        if let Some(codebase) = self.codebase {
                            let class_like_reflection = match &object_kind {
                                ObjectTypeKind::NamedObject { name, .. } => codebase.get_named_class_like(name),
                                ObjectTypeKind::AnonymousObject { span } => {
                                    codebase.get_class_like(ClassLikeName::AnonymousClass(*span))
                                }
                                ObjectTypeKind::EnumCase { enum_name, .. } => codebase.get_enum(enum_name),
                                _ => {
                                    return mixed_kind(false);
                                }
                            };

                            if let Some(class_reflection) = class_like_reflection {
                                if let Some(method) = class_reflection.get_method(&method.value) {
                                    return method.return_type_reflection.as_ref().map_or_else(
                                        || mixed_kind(false),
                                        |return_type| return_type.type_reflection.kind.clone(),
                                    );
                                }
                            }
                        }
                    }

                    mixed_kind(false)
                }
                Call::StaticMethod(static_method_call) => {
                    if let Some(codebase) = self.codebase {
                        if let (Expression::Identifier(name), ClassLikeMemberSelector::Identifier(method)) =
                            (static_method_call.class.as_ref(), &static_method_call.method)
                        {
                            let class_name = self.names.get(name);

                            if let Some(class_reflection) = codebase.get_named_class_like(class_name) {
                                if let Some(method) = class_reflection.get_method(&method.value) {
                                    return method.return_type_reflection.as_ref().map_or_else(
                                        || mixed_kind(false),
                                        |return_type| return_type.type_reflection.kind.clone(),
                                    );
                                }
                            }
                        }
                    }

                    mixed_kind(false)
                }
            },
            Expression::Access(access) => match access.as_ref() {
                Access::Property(property_access) => {
                    let object_kind = self.resolve(&property_access.object);

                    if let TypeKind::Object(object_kind) = object_kind {
                        let ClassLikeMemberSelector::Identifier(property) = &property_access.property else {
                            return mixed_kind(false);
                        };

                        if let Some(codebase) = self.codebase {
                            let class_like_reflection = match &object_kind {
                                ObjectTypeKind::NamedObject { name, .. } => codebase.get_named_class_like(name),
                                ObjectTypeKind::AnonymousObject { span } => {
                                    codebase.get_class_like(ClassLikeName::AnonymousClass(*span))
                                }
                                ObjectTypeKind::EnumCase { enum_name, .. } => codebase.get_enum(enum_name),
                                _ => {
                                    return mixed_kind(false);
                                }
                            };

                            let property = self.interner.intern(format!("${}", self.interner.lookup(&property.value)));
                            if let Some(class_reflection) = class_like_reflection {
                                if let Some(property) = class_reflection.get_property(&property) {
                                    return property
                                        .type_reflection
                                        .as_ref()
                                        .map(|t| t.kind.clone())
                                        .or_else(|| {
                                            property
                                                .default_value_reflection
                                                .as_ref()
                                                .map(|v| v.inferred_type_reflection.kind.clone())
                                        })
                                        .unwrap_or_else(|| mixed_kind(false));
                                }
                            }
                        }
                    }

                    mixed_kind(false)
                }
                Access::NullSafeProperty(null_safe_property_access) => {
                    let object_kind = self.resolve(&null_safe_property_access.object);

                    if let TypeKind::Object(object_kind) = object_kind {
                        let ClassLikeMemberSelector::Identifier(property) = &null_safe_property_access.property else {
                            return mixed_kind(false);
                        };

                        if let Some(codebase) = self.codebase {
                            let class_like_reflection = match &object_kind {
                                ObjectTypeKind::NamedObject { name, .. } => codebase.get_named_class_like(name),
                                ObjectTypeKind::AnonymousObject { span } => {
                                    codebase.get_class_like(ClassLikeName::AnonymousClass(*span))
                                }
                                ObjectTypeKind::EnumCase { enum_name, .. } => codebase.get_enum(enum_name),
                                _ => {
                                    return mixed_kind(false);
                                }
                            };

                            let property = self.interner.intern(format!("${}", self.interner.lookup(&property.value)));
                            if let Some(class_reflection) = class_like_reflection {
                                if let Some(property) = class_reflection.get_property(&property) {
                                    return property
                                        .type_reflection
                                        .as_ref()
                                        .map(|t| t.kind.clone())
                                        .or_else(|| {
                                            property
                                                .default_value_reflection
                                                .as_ref()
                                                .map(|v| v.inferred_type_reflection.kind.clone())
                                        })
                                        .unwrap_or_else(|| mixed_kind(false));
                                }
                            }
                        }
                    }

                    mixed_kind(false)
                }
                Access::StaticProperty(static_property_access) => {
                    if let Some(codebase) = self.codebase {
                        if let (Expression::Identifier(name), Variable::Direct(variable)) =
                            (&static_property_access.class, &static_property_access.property)
                        {
                            let class_name = self.names.get(name);

                            if let Some(class_reflection) = codebase.get_named_class_like(class_name) {
                                if let Some(property) = class_reflection.get_property(&variable.name) {
                                    return property
                                        .type_reflection
                                        .as_ref()
                                        .map(|t| t.kind.clone())
                                        .or_else(|| {
                                            property
                                                .default_value_reflection
                                                .as_ref()
                                                .map(|v| v.inferred_type_reflection.kind.clone())
                                        })
                                        .unwrap_or_else(|| mixed_kind(false));
                                }
                            }
                        }
                    }

                    mixed_kind(false)
                }
                Access::ClassConstant(class_constant_access) => {
                    if let Some(codebase) = self.codebase {
                        if let (Expression::Identifier(name), ClassLikeConstantSelector::Identifier(constant)) =
                            (&class_constant_access.class, &class_constant_access.constant)
                        {
                            let class_name = self.names.get(name);

                            if let Some(class_reflection) = codebase.get_named_class_like(class_name) {
                                if let Some(constant) = class_reflection.get_constant(&constant.value) {
                                    return constant
                                        .type_reflection
                                        .as_ref()
                                        .map(|t| t.kind.clone())
                                        .unwrap_or_else(|| constant.inferred_type_reflection.kind.clone());
                                }

                                if class_reflection.is_enum() && class_reflection.has_enum_case(&constant.value) {
                                    return enum_case_kind(*class_name, constant.value);
                                }
                            }
                        }
                    }

                    mixed_kind(false)
                }
            },
            Expression::ClosureCreation(closure_creation) => match closure_creation.as_ref() {
                ClosureCreation::Function(function_closure_creation) => {
                    if let Some(codebase) = &self.codebase {
                        if let Expression::Identifier(name) = &function_closure_creation.function {
                            let (full_name, short_name) = resolve_name(self.interner, name.value());

                            if let Some(function) = codebase.get_function(&full_name) {
                                return TypeKind::from(function);
                            }

                            // fallback to short name, welcome to PHP.
                            if let Some(function) = codebase.get_function(&short_name) {
                                return TypeKind::from(function);
                            }
                        }
                    }

                    if let TypeKind::Callable(callable) = self.resolve(&function_closure_creation.function) {
                        match callable {
                            CallableTypeKind::Callable { pure, templates, parameters, return_kind } => {
                                return TypeKind::Callable(CallableTypeKind::Closure {
                                    pure,
                                    templates,
                                    parameters,
                                    return_kind,
                                });
                            }
                            closure @ CallableTypeKind::Closure { .. } => {
                                return TypeKind::Callable(closure);
                            }
                        }
                    }

                    any_closure_kind()
                }
                ClosureCreation::Method(method_closure_creation) => {
                    if let Some(codebase) = &self.codebase {
                        let ClassLikeMemberSelector::Identifier(method_name) = &method_closure_creation.method else {
                            return any_closure_kind();
                        };

                        let TypeKind::Object(object_kind) = self.resolve(&method_closure_creation.object) else {
                            return any_closure_kind();
                        };

                        let class_reflection = match object_kind {
                            ObjectTypeKind::NamedObject { name, .. } => {
                                if let Some(class_like) = codebase.get_named_class_like(&name) {
                                    class_like
                                } else {
                                    return any_closure_kind();
                                }
                            }
                            ObjectTypeKind::EnumCase { enum_name, .. } => {
                                if let Some(class_like) = codebase.get_enum(&enum_name) {
                                    class_like
                                } else {
                                    return any_closure_kind();
                                }
                            }
                            ObjectTypeKind::AnonymousObject { span } => {
                                if let Some(class) = codebase.get_class_like(ClassLikeName::AnonymousClass(span)) {
                                    class
                                } else {
                                    return any_closure_kind();
                                }
                            }
                            _ => return any_closure_kind(),
                        };

                        if let Some(method) = class_reflection.get_method(&method_name.value) {
                            return TypeKind::from(method);
                        } else {
                            return any_closure_kind();
                        }
                    }

                    any_closure_kind()
                }
                ClosureCreation::StaticMethod(static_method_closure_creation) => {
                    if let Some(codebase) = &self.codebase {
                        let Expression::Identifier(class_name) = &static_method_closure_creation.class else {
                            return any_closure_kind();
                        };

                        let ClassLikeMemberSelector::Identifier(method_name) = &static_method_closure_creation.method
                        else {
                            return any_closure_kind();
                        };

                        let class_name = self.names.get(class_name);
                        let Some(class_reflection) = codebase.get_class(class_name) else {
                            return any_closure_kind();
                        };

                        if let Some(method) = class_reflection.get_method(&method_name.value) {
                            return TypeKind::from(method);
                        } else {
                            return any_closure_kind();
                        }
                    }

                    any_closure_kind()
                }
            },
            Expression::Parent(_) => TypeKind::Scalar(ScalarTypeKind::ClassString(None)),
            Expression::Static(_) => TypeKind::Scalar(ScalarTypeKind::ClassString(None)),
            Expression::Self_(_) => TypeKind::Scalar(ScalarTypeKind::ClassString(None)),
            Expression::Instantiation(instantiation) => {
                let Expression::Identifier(class_name) = &instantiation.class else {
                    return any_object_kind();
                };

                let (class_name, _) = resolve_name(self.interner, class_name.value());

                TypeKind::Object(ObjectTypeKind::NamedObject { name: class_name, type_parameters: vec![] })
            }
            Expression::MagicConstant(magic_constant) => match &magic_constant {
                MagicConstant::Line(local_identifier) => {
                    let line = self.source.line_number(local_identifier.offset());

                    value_integer_kind(line as i64)
                }
                MagicConstant::File(_) => {
                    if let Some(file) = &self.source.path {
                        let file_id = self.interner.intern(file.to_string_lossy());

                        get_literal_string_value_kind(self.interner, file_id, false)
                    } else {
                        non_empty_string_kind()
                    }
                }
                MagicConstant::Directory(_) => {
                    if let Some(directory) = self.source.path.as_ref().and_then(|p| p.parent()) {
                        let directory_id = self.interner.intern(directory.to_string_lossy());

                        get_literal_string_value_kind(self.interner, directory_id, false)
                    } else {
                        non_empty_string_kind()
                    }
                }
                MagicConstant::Trait(_) => union_kind(vec![
                    TypeKind::Scalar(ScalarTypeKind::TraitString),
                    value_string_kind(
                        StringIdentifier::empty(),
                        0,
                        Trinary::False,
                        Trinary::False,
                        Trinary::False,
                        Trinary::False,
                    ),
                ]),
                MagicConstant::Method(_)
                | MagicConstant::Function(_)
                | MagicConstant::Property(_)
                | MagicConstant::Namespace(_) => TypeKind::Scalar(ScalarTypeKind::LiteralString),
                MagicConstant::Class(_) => TypeKind::Scalar(ScalarTypeKind::ClassString(None)),
            },
            // Non-readable expressions
            Expression::ArrayAppend(_) => never_kind(),
            Expression::List(_) => never_kind(),
            // Requires more context
            _ => mixed_kind(false),
        }
    }
}
