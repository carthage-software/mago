use mago_atom::Atom;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::identifier::method::MethodIdentifier;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_codex::metadata::function_like::TemplateTuple;
use mago_codex::metadata::parameter::FunctionLikeParameterMetadata;
use mago_codex::misc::VariableIdentifier;
use mago_codex::ttype::atomic::callable::TCallableSignature;
use mago_codex::ttype::atomic::callable::parameter::TCallableParameter;
use mago_codex::ttype::expander::StaticClassType;
use mago_codex::ttype::union::TUnion;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

mod arguments;
mod resolver;
mod special_function_like_handler;
mod template_inference;

pub mod analyzer;
pub mod post_process;
pub mod return_type_fetcher;
pub mod template_result;

/// Represents a resolved invocation of a function, method, or any callable expression.
///
/// This struct captures all necessary information about a call site, including
/// what is being called (`target`), the arguments passed (`arguments`), and the
/// source span of the entire invocation.
#[derive(Debug, Clone)]
pub struct Invocation<'ctx, 'ast, 'arena> {
    /// The resolved target of the call, which could be a named function/method
    /// or a dynamic callable expression (e.g., a closure, an invocable object).
    pub target: InvocationTarget<'ctx>,
    /// The arguments provided to the call, either as a standard argument list
    /// or as the input from a pipe operator.
    pub arguments_source: InvocationArgumentsSource<'ast, 'arena>,
    /// The source code span covering the entire invocation expression (e.g., `func(arg)` or `$val |> func`).
    pub span: Span,
}

/// Holds contextual information specific to a method call resolution.
///
/// When a method is invoked (e.g., `$obj->method()` or `ParentClass::method()`),
/// this struct provides details about the class context in which the call occurs
/// and how the method was resolved.
#[derive(Debug, Clone)]
pub struct MethodTargetContext<'ctx> {
    /// The specific `MethodIdentifier` (class name + method name) of the method
    /// that will be invoked, if statically resolved. This points to the method's
    /// declaration, which might be in a parent class or trait.
    pub declaring_method_id: Option<MethodIdentifier>,
    /// Metadata for the class context (`self_fq_class_like_name`).
    pub class_like_metadata: &'ctx ClassLikeMetadata,
    /// The type of the class context, which is used to resolve `static::class` and
    pub class_type: StaticClassType,
}

/// Represents the target of an invocation, distinguishing between statically known
/// functions/methods and dynamic callable expressions.
///
/// This allows the analyzer to use specific metadata for known functions/methods
/// or rely on the `TCallableSignature` for dynamic callables.
#[derive(Debug, Clone)]
pub enum InvocationTarget<'ctx> {
    /// The invocation target is a dynamic callable whose exact identity isn't known
    /// until runtime, but its signature (parameters, return type) is known.
    /// Examples include closures, invocable objects, or variables holding callables.
    Callable {
        /// If the callable expression could be traced back to an original named function
        /// or method (e.g., `$callable = strlen(...); $callable()`), this might hold its identifier.
        source: Option<FunctionLikeIdentifier>,
        /// The type signature (`(param_types) => return_type`) of the callable.
        signature: TCallableSignature,
        /// The span of the expression that evaluates to this callable (e.g., span of `$var` in `$var()`).
        span: Span,
    },
    /// The invocation target is a statically resolved function or method.
    FunctionLike {
        /// The unique identifier for the statically resolved function or method.
        identifier: FunctionLikeIdentifier,
        /// Metadata (parameters, return type, etc.) for the resolved function or method.
        metadata: &'ctx FunctionLikeMetadata,
        /// The inferred return type for this function-like.
        ///
        /// This type is usually set for closures and arrow functions, and
        /// contains a more precise return type than the declared one.
        inferred_return_type: Option<Box<TUnion>>,
        /// If this is a method call, this provides context about the calling class
        /// (e.g., type of `$this`, resolved `static::class`). `None` for function calls.
        method_context: Option<MethodTargetContext<'ctx>>,
        /// The span of the callable part of the invocation expression
        /// (e.g., `my_function` in `my_function(...)` or `$obj->myMethod` in `$obj->myMethod(...)`).
        span: Span,
    },
}

/// Represents a parameter definition, abstracting over parameters from statically
/// known functions/methods and parameters from dynamic `TCallableSignature`s.
///
/// This allows argument checking logic to treat both sources of parameter information
/// uniformly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvocationTargetParameter<'ctx> {
    /// Parameter from a statically defined function or method.
    FunctionLike(&'ctx FunctionLikeParameterMetadata),
    /// Parameter from a `TCallableSignature` (e.g., from a closure type or `callable` type hint).
    Callable(&'ctx TCallableParameter),
}

/// Represents the source of arguments for an invocation.
///
/// This distinguishes between standard argument lists `func(args)` and
/// arguments provided via the pipe operator `$input |> func`.
#[derive(Debug, Clone, Copy)]
pub enum InvocationArgumentsSource<'ast, 'arena> {
    /// No arguments are present, e.g., calling `__construct` via `new Foo`,
    /// or `__toString` via `(string) $foo`.
    None(Span),
    /// Arguments are provided in a standard list, like `foo($a, $b)`.
    ArgumentList(&'ast ArgumentList<'arena>),
    /// The single argument is the input from a pipe operator, like `$input` in `$input |> foo(...)`.
    PipeInput(&'ast Pipe<'arena>),
}

/// Represents a single argument passed during an invocation, abstracting whether
/// it's a standard argument or a value piped in.
///
/// This allows iteration over "effective arguments" regardless of how they were supplied.
#[derive(Debug, Clone, Copy)]
pub enum InvocationArgument<'ast, 'arena> {
    /// A standard argument from an `ArgumentList`.
    Argument(&'ast Argument<'arena>),
    /// The value provided as input via the pipe operator. This is treated as the first positional argument.
    PipedValue(&'ast Expression<'arena>),
}

impl<'ctx, 'ast, 'arena> Invocation<'ctx, 'ast, 'arena> {
    pub fn new(target: InvocationTarget<'ctx>, arguments: InvocationArgumentsSource<'ast, 'arena>, span: Span) -> Self {
        Self { target, arguments_source: arguments, span }
    }
}

impl<'ctx> InvocationTarget<'ctx> {
    /// Attempts to guess a human-readable name for the callable target.
    ///
    /// Returns the name of a function/method if statically known,
    /// or "Closure" or "callable" for dynamic callables.
    pub fn guess_name(&self) -> String {
        self.get_function_like_identifier().map(|id| id.as_string()).unwrap_or_else(|| {
            if self.is_non_closure_callable() { "callable".to_string() } else { "Closure".to_string() }
        })
    }

    /// Guesses the kind of the callable target (e.g., "function", "method", "closure", "callable").
    pub fn guess_kind(&self) -> &'static str {
        match self.get_function_like_identifier() {
            Some(identifier) => match identifier {
                FunctionLikeIdentifier::Function(_) => "function",
                FunctionLikeIdentifier::Method(_, _) => "method",
                FunctionLikeIdentifier::Closure(_, _) => "closure",
            },
            None => {
                if self.is_non_closure_callable() {
                    "callable"
                } else {
                    "closure"
                }
            }
        }
    }

    pub const fn is_method_call(&self) -> bool {
        matches!(self.get_function_like_identifier(), Some(FunctionLikeIdentifier::Method(_, _)))
    }

    /// Checks if the target is a dynamic callable that is not explicitly a closure type.
    /// This can be true for `callable` type hints or invocable objects that aren't closures.
    #[inline]
    pub const fn is_non_closure_callable(&self) -> bool {
        match self {
            InvocationTarget::Callable { signature, .. } => !signature.is_closure(),
            _ => false,
        }
    }

    /// Returns the metadata if this target is a statically known function or method.
    #[inline]
    pub const fn get_function_like_metadata(&self) -> Option<&'ctx FunctionLikeMetadata> {
        match self {
            InvocationTarget::FunctionLike { metadata, .. } => Some(metadata),
            _ => None,
        }
    }

    /// Returns the `FunctionLikeIdentifier` if available (for static functions/methods or traced callables).
    #[inline]
    pub const fn get_function_like_identifier(&self) -> Option<&FunctionLikeIdentifier> {
        match self {
            InvocationTarget::Callable { source, .. } => source.as_ref(),
            InvocationTarget::FunctionLike { identifier, .. } => Some(identifier),
        }
    }

    /// If this target is a method, returns the fully qualified name of the class it belongs to.
    #[inline]
    #[allow(dead_code)]
    pub const fn get_method_class_like_name(&self) -> Option<Atom> {
        match self.get_function_like_identifier() {
            Some(FunctionLikeIdentifier::Method(fq_class_like_name, _)) => Some(*fq_class_like_name),
            _ => None,
        }
    }

    /// If this target is a method, returns its `MethodIdentifier`.
    #[inline]
    #[allow(dead_code)]
    pub const fn get_method_identifier(&self) -> Option<MethodIdentifier> {
        match self {
            InvocationTarget::FunctionLike { identifier, .. } => identifier.as_method_identifier(),
            _ => None,
        }
    }

    /// Checks if the target function/method is known to potentially throw exceptions (e.g., has `@throws` tags).
    #[inline]
    #[allow(dead_code)]
    pub const fn has_throw(&self) -> bool {
        match self {
            InvocationTarget::FunctionLike { metadata, .. } => metadata.flags.has_throw(),
            _ => false,
        }
    }

    /// Returns the template type definitions if the target is a generic function or method.
    #[inline]
    pub fn get_template_types(&self) -> Option<&'ctx [TemplateTuple]> {
        match self {
            InvocationTarget::FunctionLike { metadata, .. } => Some(&metadata.template_types),
            _ => None,
        }
    }

    /// Checks if the target function/method allows named arguments.
    #[inline]
    pub const fn allows_named_arguments(&self) -> bool {
        match self {
            InvocationTarget::FunctionLike { metadata, .. } => !metadata.flags.forbids_named_arguments(),
            _ => false,
        }
    }

    /// Returns the `MethodTargetContext` if this invocation is a method call.
    #[inline]
    pub const fn get_method_context(&self) -> Option<&MethodTargetContext<'ctx>> {
        match self {
            InvocationTarget::FunctionLike { method_context, .. } => method_context.as_ref(),
            _ => None,
        }
    }

    /// Retrieves a list of parameters for the invocation target.
    ///
    /// Parameters are wrapped in `InvocationTargetParameter` to abstract over
    /// `FunctionLikeParameterMetadata` and `TCallableParameter`.
    #[inline]
    pub fn get_parameters<'target>(&'target self) -> Vec<InvocationTargetParameter<'target>>
    where
        'ctx: 'target,
    {
        match self {
            InvocationTarget::Callable { signature, .. } => {
                signature.parameters.iter().map(InvocationTargetParameter::Callable).collect()
            }
            InvocationTarget::FunctionLike { metadata, .. } => {
                metadata.parameters.iter().map(InvocationTargetParameter::FunctionLike).collect()
            }
        }
    }

    /// Retrieves the return type of the invocation target, if known.
    #[inline]
    pub fn get_return_type(&self) -> Option<&TUnion> {
        match self {
            InvocationTarget::Callable { signature, .. } => signature.get_return_type(),
            InvocationTarget::FunctionLike { metadata, inferred_return_type, .. } => inferred_return_type
                .as_deref()
                .or_else(|| metadata.return_type_metadata.as_ref().map(|type_metadata| &type_metadata.type_union)),
        }
    }
}

impl<'a> InvocationTargetParameter<'a> {
    /// Gets the type (`TUnion`) of the parameter.
    #[inline]
    pub fn get_out_type(&self) -> Option<&'a TUnion> {
        match self {
            InvocationTargetParameter::FunctionLike(metadata) => {
                metadata.out_type.as_ref().map(|type_metadata| &type_metadata.type_union)
            }
            _ => None,
        }
    }

    /// Gets the type (`TUnion`) of the parameter.
    #[inline]
    pub fn get_type(&self) -> Option<&'a TUnion> {
        match self {
            InvocationTargetParameter::FunctionLike(metadata) => {
                metadata.get_type_metadata().map(|type_metadata| &type_metadata.type_union)
            }
            InvocationTargetParameter::Callable(parameter) => parameter.get_type_signature(),
        }
    }

    /// Gets the name of the parameter as a `VariableIdentifier`, if available
    /// (primarily for `FunctionLike` parameters).
    #[inline]
    pub fn get_name(&self) -> Option<&'a VariableIdentifier> {
        // Changed to &'a
        match self {
            InvocationTargetParameter::FunctionLike(metadata) => Some(metadata.get_name()),
            InvocationTargetParameter::Callable(_) => None,
        }
    }

    /// Checks if the parameter is passed by reference (`&`).
    #[inline]
    #[allow(dead_code)]
    pub const fn is_by_reference(&self) -> bool {
        match self {
            InvocationTargetParameter::FunctionLike(metadata) => metadata.flags.is_by_reference(),
            InvocationTargetParameter::Callable(parameter) => parameter.is_by_reference(),
        }
    }

    /// Checks if the parameter is variadic (`...`).
    #[inline]
    pub const fn is_variadic(&self) -> bool {
        match self {
            InvocationTargetParameter::FunctionLike(metadata) => metadata.flags.is_variadic(),
            InvocationTargetParameter::Callable(parameter) => parameter.is_variadic(),
        }
    }

    /// Checks if the parameter has a default value.
    #[inline]
    pub const fn has_default(&self) -> bool {
        match self {
            InvocationTargetParameter::FunctionLike(metadata) => metadata.flags.has_default(),
            InvocationTargetParameter::Callable(parameter) => parameter.has_default(),
        }
    }

    /// Get the default value type for the parameter
    #[inline]
    pub fn get_default_type(&self) -> Option<&'a TUnion> {
        match self {
            InvocationTargetParameter::FunctionLike(metadata) => {
                metadata.get_default_type().map(|type_metadata| &type_metadata.type_union)
            }
            InvocationTargetParameter::Callable(_) => None,
        }
    }
}

impl<'ast, 'arena> InvocationArgumentsSource<'ast, 'arena> {
    /// Returns a `Vec` of `InvocationArgument` which abstracts over standard arguments
    /// and piped input. For pipe input, it's a single `PipedValue`.
    #[inline]
    pub fn get_arguments(&self) -> Vec<InvocationArgument<'ast, 'arena>> {
        match self {
            InvocationArgumentsSource::ArgumentList(arg_list) => {
                arg_list.arguments.iter().map(InvocationArgument::Argument).collect()
            }
            InvocationArgumentsSource::PipeInput(pipe) => {
                vec![InvocationArgument::PipedValue(pipe.input)]
            }
            InvocationArgumentsSource::None(_) => {
                vec![]
            }
        }
    }
}

impl<'ast, 'arena> InvocationArgument<'ast, 'arena> {
    /// Checks if this argument is positional (not named).
    /// Piped values are considered positional.
    #[inline]
    pub const fn is_positional(&self) -> bool {
        match self {
            InvocationArgument::Argument(arg) => arg.is_positional(),
            _ => true,
        }
    }

    /// Checks if this argument is an unpacked argument (`...$args`).
    /// Piped values cannot be unpacked in this way.
    #[inline]
    pub const fn is_unpacked(&self) -> bool {
        match self {
            InvocationArgument::Argument(arg) => arg.is_unpacked(),
            _ => false,
        }
    }

    /// Returns a reference to the underlying `Expression` of the argument's value.
    #[inline]
    pub const fn value(&self) -> &'ast Expression<'arena> {
        match self {
            InvocationArgument::Argument(arg) => arg.value(),
            InvocationArgument::PipedValue(expr) => expr,
        }
    }

    /// If this argument is a standard named argument, returns a reference to it.
    /// Returns `None` for positional arguments or piped values.
    #[inline]
    pub const fn get_named_argument(&self) -> Option<&'ast NamedArgument<'arena>> {
        match self {
            InvocationArgument::Argument(arg) => match arg {
                Argument::Named(named_arg) => Some(named_arg),
                Argument::Positional(_) => None,
            },
            _ => None,
        }
    }
}

impl HasSpan for Invocation<'_, '_, '_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for InvocationTarget<'_> {
    fn span(&self) -> Span {
        match self {
            InvocationTarget::Callable { span, .. } => *span,
            InvocationTarget::FunctionLike { span, .. } => *span,
        }
    }
}

impl HasSpan for InvocationArgumentsSource<'_, '_> {
    fn span(&self) -> Span {
        match self {
            InvocationArgumentsSource::ArgumentList(arg_list) => arg_list.span(),
            InvocationArgumentsSource::PipeInput(pipe) => pipe.span(),
            InvocationArgumentsSource::None(span) => *span,
        }
    }
}

impl HasSpan for InvocationArgument<'_, '_> {
    fn span(&self) -> Span {
        match self {
            InvocationArgument::Argument(arg) => arg.span(),
            InvocationArgument::PipedValue(expr) => expr.span(),
        }
    }
}
