use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::identifier::method::MethodIdentifier;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_codex::metadata::function_like::TemplateTuple;
use mago_codex::metadata::parameter::FunctionLikeParameterMetadata;
use mago_codex::misc::VariableIdentifier;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::callable::TCallableSignature;
use mago_codex::ttype::atomic::callable::parameter::TCallableParameter;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::expander::StaticClassType;
use mago_codex::ttype::get_literal_int;
use mago_codex::ttype::get_never;
use mago_codex::ttype::get_scalar;
use mago_codex::ttype::get_string;
use mago_codex::ttype::get_void;
use mago_codex::ttype::union::TUnion;
use mago_interner::StringIdentifier;
use mago_interner::ThreadedInterner;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

mod resolver;
mod special_function_like_handler;

pub mod analyzer;
pub mod post_process;
pub mod return_type_fetcher;

/// Enumerates specific PHP language constructs that can be analyzed like function calls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LanguageConstructKind {
    Echo,
    Print,
    Exit,
}

impl LanguageConstructKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            LanguageConstructKind::Echo => "echo",
            LanguageConstructKind::Print => "print",
            LanguageConstructKind::Exit => "exit",
        }
    }

    /// Checks if the construct supports variadic arguments.
    pub fn is_varadic(&self) -> bool {
        match self {
            LanguageConstructKind::Echo => true,
            LanguageConstructKind::Print => false,
            LanguageConstructKind::Exit => false,
        }
    }

    /// Checks if the construct parameter is optional.
    pub fn has_default(&self) -> bool {
        match self {
            LanguageConstructKind::Echo => false,
            LanguageConstructKind::Print => false,
            LanguageConstructKind::Exit => true,
        }
    }

    /// Gets the effective parameters for this construct.
    /// This allows the argument checking logic to be reused.
    pub fn get_parameter_type(&self) -> TUnion {
        match self {
            LanguageConstructKind::Echo => get_scalar().as_nullable(),
            LanguageConstructKind::Print => get_string(),
            LanguageConstructKind::Exit => {
                TUnion::new(vec![TAtomic::Scalar(TScalar::int()), TAtomic::Scalar(TScalar::string())])
            }
        }
    }

    /// Gets the return type of this construct.
    pub fn get_return_type(&self) -> TUnion {
        match self {
            LanguageConstructKind::Echo => get_void(),
            LanguageConstructKind::Print => get_literal_int(1),
            LanguageConstructKind::Exit => get_never(),
        }
    }
}

/// Represents a resolved invocation of a function, method, or any callable expression.
///
/// This struct captures all necessary information about a call site, including
/// what is being called (`target`), the arguments passed (`arguments`), and the
/// source span of the entire invocation.
#[derive(Debug, Clone)]
pub struct Invocation<'a> {
    /// The resolved target of the call, which could be a named function/method
    /// or a dynamic callable expression (e.g., a closure, an invokable object).
    pub target: InvocationTarget<'a>,
    /// The arguments provided to the call, either as a standard argument list
    /// or as the input from a pipe operator.
    pub arguments_source: InvocationArgumentsSource<'a>,
    /// The source code span covering the entire invocation expression (e.g., `func(arg)` or `$val |> func`).
    pub span: Span,
}

/// Holds contextual information specific to a method call resolution.
///
/// When a method is invoked (e.g., `$obj->method()` or `ParentClass::method()`),
/// this struct provides details about the class context in which the call occurs
/// and how the method was resolved.
#[derive(Debug, Clone)]
pub struct MethodTargetContext<'a> {
    /// The specific `MethodIdentifier` (class name + method name) of the method
    /// that will be invoked, if statically resolved. This points to the method's
    /// declaration, which might be in a parent class or trait.
    pub declaring_method_id: Option<MethodIdentifier>,
    /// Metadata for the class context (`self_fq_class_like_name`).
    pub class_like_metadata: &'a ClassLikeMetadata,
    /// The type of the class context, which is used to resolve `static::class` and
    pub class_type: StaticClassType,
}

/// Represents the target of an invocation, distinguishing between statically known
/// functions/methods and dynamic callable expressions.
///
/// This allows the analyzer to use specific metadata for known functions/methods
/// or rely on the `TCallableSignature` for dynamic callables.
#[derive(Debug, Clone)]
pub enum InvocationTarget<'a> {
    /// The invocation target is a dynamic callable whose exact identity isn't known
    /// until runtime, but its signature (parameters, return type) is known.
    /// Examples include closures, invokable objects, or variables holding callables.
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
        metadata: &'a FunctionLikeMetadata,
        /// If this is a method call, this provides context about the calling class
        /// (e.g., type of `$this`, resolved `static::class`). `None` for function calls.
        method_context: Option<MethodTargetContext<'a>>,
        /// The span of the callable part of the invocation expression
        /// (e.g., `my_function` in `my_function(...)` or `$obj->myMethod` in `$obj->myMethod(...)`).
        span: Span,
    },
    /// The invocation target is a language construct (e.g., `echo`, `print`).
    LanguageConstruct {
        /// The kind of the construct (e.g., `echo`, `print`).
        kind: LanguageConstructKind,
        /// The parameter for the construct, which is a `TCallableParameter`.
        parameter: TCallableParameter,
        /// The return type of the construct, which is a `TUnion`.
        return_type: TUnion,
        /// The span of the construct in the source code.
        span: Span,
    },
}

/// Represents a parameter definition, abstracting over parameters from statically
/// known functions/methods and parameters from dynamic `TCallableSignature`s.
///
/// This allows argument checking logic to treat both sources of parameter information
/// uniformly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvocationTargetParameter<'a> {
    /// Parameter from a statically defined function or method.
    FunctionLike(&'a FunctionLikeParameterMetadata),
    /// Parameter from a `TCallableSignature` (e.g., from a closure type or `callable` type hint).
    Callable(&'a TCallableParameter),
}

/// Represents the source of arguments for an invocation.
///
/// This distinguishes between standard argument lists `func(args)` and
/// arguments provided via the pipe operator `$input |> func`.
#[derive(Debug, Clone, Copy)]
pub enum InvocationArgumentsSource<'a> {
    /// No arguments are present, e.g., calling `__construct` via `new Foo`,
    /// or `__toString` via `(string) $foo`.
    None(Span),
    /// Arguments are provided in a standard list, like `foo($a, $b)`.
    ArgumentList(&'a ArgumentList),
    /// The single argument is the input from a pipe operator, like `$input` in `$input |> foo(...)`.
    PipeInput(&'a Pipe),
    /// A slice of expressions, used for constructs like `echo` or `print`.
    Slice(&'a [Expression]),
}

/// Represents a single argument passed during an invocation, abstracting whether
/// it's a standard argument or a value piped in.
///
/// This allows iteration over "effective arguments" regardless of how they were supplied.
#[derive(Debug, Clone, Copy)]
pub enum InvocationArgument<'a> {
    /// A standard argument from an `ArgumentList`.
    Argument(&'a Argument),
    /// The value provided as input via the pipe operator. This is treated as the first positional argument.
    PipedValue(&'a Expression),
    /// A single expression, used for constructs like `echo`.
    Expression(&'a Expression),
}

impl<'a> Invocation<'a> {
    /// Creates a new `Invocation` instance.
    pub fn new(target: InvocationTarget<'a>, arguments: InvocationArgumentsSource<'a>, span: Span) -> Self {
        Self { target, arguments_source: arguments, span }
    }
}

impl<'a> InvocationTarget<'a> {
    pub fn for_language_construct(kind: LanguageConstructKind, span: Span) -> Self {
        let parameter = TCallableParameter::new(
            Some(Box::new(kind.get_parameter_type())),
            false,
            kind.is_varadic(),
            kind.has_default(),
        );

        Self::LanguageConstruct { kind, parameter, return_type: kind.get_return_type(), span }
    }

    /// Attempts to guess a human-readable name for the callable target.
    ///
    /// Returns the name of a function/method if statically known,
    /// or "Closure" or "callable" for dynamic callables.
    pub fn guess_name(&self, interner: &ThreadedInterner) -> String {
        self.get_function_like_identifier().map(|id| id.as_string(interner)).unwrap_or_else(|| {
            if let InvocationTarget::LanguageConstruct { kind, .. } = self {
                kind.as_str().to_string()
            } else if self.is_non_closure_callable() {
                "callable".to_string()
            } else {
                "Closure".to_string()
            }
        })
    }

    /// Guesses the kind of the callable target (e.g., "function", "method", "closure", "callable").
    pub fn guess_kind(&self) -> &'static str {
        match self.get_function_like_identifier() {
            Some(identifier) => match identifier {
                FunctionLikeIdentifier::Function(_) => "function",
                FunctionLikeIdentifier::Method(_, _) => "method",
                FunctionLikeIdentifier::Closure(_) => "closure",
            },
            None => {
                if self.is_language_construct() {
                    "construct"
                } else if self.is_non_closure_callable() {
                    "callable"
                } else {
                    "closure"
                }
            }
        }
    }

    /// Check if the target is a language construct (e.g., `echo`, `print`).
    #[inline]
    pub const fn is_language_construct(&self) -> bool {
        matches!(self, InvocationTarget::LanguageConstruct { .. })
    }

    /// Checks if the target is a dynamic callable that is not explicitly a closure type.
    /// This can be true for `callable` type hints or invokable objects that aren't closures.
    #[inline]
    pub const fn is_non_closure_callable(&self) -> bool {
        match self {
            InvocationTarget::Callable { signature, .. } => !signature.is_closure(),
            _ => false,
        }
    }

    /// Returns the metadata if this target is a statically known function or method.
    #[inline]
    pub const fn get_function_like_metadata(&self) -> Option<&'a FunctionLikeMetadata> {
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
            _ => None,
        }
    }

    /// If this target is a method, returns the fully qualified name of the class it belongs to.
    #[inline]
    #[allow(dead_code)]
    pub const fn get_method_class_like_name(&self) -> Option<&StringIdentifier> {
        match self.get_function_like_identifier() {
            Some(FunctionLikeIdentifier::Method(fq_class_like_name, _)) => Some(fq_class_like_name),
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
            InvocationTarget::FunctionLike { metadata, .. } => metadata.has_throw,
            _ => false,
        }
    }

    /// Returns the template type definitions if the target is a generic function or method.
    #[inline]
    pub fn get_template_types(&self) -> Option<&'a [TemplateTuple]> {
        match self {
            InvocationTarget::FunctionLike { metadata, .. } => Some(metadata.get_template_types()),
            _ => None,
        }
    }

    /// Checks if the target function/method allows named arguments.
    #[inline]
    pub const fn allows_named_arguments(&self) -> bool {
        match self {
            InvocationTarget::FunctionLike { metadata, .. } => metadata.allows_named_arguments,
            _ => false,
        }
    }

    /// Returns the `MethodTargetContext` if this invocation is a method call.
    #[inline]
    pub const fn get_method_context(&self) -> Option<&MethodTargetContext<'a>> {
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
    pub fn get_parameters<'c>(&'c self) -> Vec<InvocationTargetParameter<'c>>
    where
        'a: 'c, // Ensures that the lifetime 'c can't outlive 'a
    {
        match self {
            InvocationTarget::Callable { signature, .. } => {
                signature.get_parameters().iter().map(InvocationTargetParameter::Callable).collect()
            }
            InvocationTarget::FunctionLike { metadata, .. } => {
                metadata.get_parameters().iter().map(InvocationTargetParameter::FunctionLike).collect()
            }
            InvocationTarget::LanguageConstruct { parameter, .. } => {
                vec![InvocationTargetParameter::Callable(parameter)]
            }
        }
    }

    /// Retrieves the return type of the invocation target, if known.
    #[inline]
    pub fn get_return_type(&self) -> Option<&TUnion> {
        match self {
            InvocationTarget::Callable { signature, .. } => signature.get_return_type(),
            InvocationTarget::FunctionLike { metadata, .. } => {
                metadata.get_return_type_metadata().map(|type_metadata| &type_metadata.type_union)
            }
            InvocationTarget::LanguageConstruct { return_type, .. } => Some(return_type),
        }
    }
}

impl<'a> InvocationTargetParameter<'a> {
    /// Gets the type (`TUnion`) of the parameter.
    #[inline]
    pub fn get_out_type(&self) -> Option<&'a TUnion> {
        match self {
            InvocationTargetParameter::FunctionLike(metadata) => {
                metadata.get_out_type().map(|type_metadata| &type_metadata.type_union)
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

    /// Gets the span of the parameter's name, if available
    /// (primarily for `FunctionLike` parameters).
    #[inline]
    pub fn get_name_span(&self) -> Option<Span> {
        match self {
            InvocationTargetParameter::FunctionLike(metadata) => Some(metadata.get_name_span()),
            InvocationTargetParameter::Callable(_) => None,
        }
    }

    /// Checks if the parameter is passed by reference (`&`).
    #[inline]
    #[allow(dead_code)]
    pub const fn is_by_reference(&self) -> bool {
        match self {
            InvocationTargetParameter::FunctionLike(metadata) => metadata.is_by_reference(),
            InvocationTargetParameter::Callable(parameter) => parameter.is_by_reference(),
        }
    }

    /// Checks if the parameter is variadic (`...`).
    #[inline]
    pub const fn is_variadic(&self) -> bool {
        match self {
            InvocationTargetParameter::FunctionLike(metadata) => metadata.is_variadic(),
            InvocationTargetParameter::Callable(parameter) => parameter.is_variadic(),
        }
    }

    /// Checks if the parameter has a default value.
    #[inline]
    pub const fn has_default(&self) -> bool {
        match self {
            InvocationTargetParameter::FunctionLike(metadata) => metadata.has_default(),
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

impl<'a> InvocationArgumentsSource<'a> {
    /// Returns a `Vec` of `InvocationArgument` which abstracts over standard arguments
    /// and piped input. For pipe input, it's a single `PipedValue`.
    #[inline]
    pub fn get_arguments(&self) -> Vec<InvocationArgument<'a>> {
        match self {
            InvocationArgumentsSource::ArgumentList(arg_list) => {
                arg_list.arguments.iter().map(InvocationArgument::Argument).collect()
            }
            InvocationArgumentsSource::PipeInput(pipe) => {
                vec![InvocationArgument::PipedValue(pipe.input.as_ref())]
            }
            InvocationArgumentsSource::Slice(expr_list) => {
                expr_list.iter().map(InvocationArgument::Expression).collect()
            }
            InvocationArgumentsSource::None(_) => {
                vec![]
            }
        }
    }
}

impl<'a> InvocationArgument<'a> {
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
    pub const fn value(&self) -> &'a Expression {
        match self {
            InvocationArgument::Argument(arg) => arg.value(),
            InvocationArgument::PipedValue(expr) => expr,
            InvocationArgument::Expression(expr) => expr,
        }
    }

    /// If this argument is a standard named argument, returns a reference to it.
    /// Returns `None` for positional arguments or piped values.
    #[inline]
    pub const fn get_named_argument(&self) -> Option<&NamedArgument> {
        match self {
            InvocationArgument::Argument(arg) => match arg {
                Argument::Named(named_arg) => Some(named_arg),
                Argument::Positional(_) => None,
            },
            _ => None,
        }
    }
}

impl HasSpan for Invocation<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for InvocationTarget<'_> {
    fn span(&self) -> Span {
        match self {
            InvocationTarget::Callable { span, .. } => *span,
            InvocationTarget::FunctionLike { span, .. } => *span,
            InvocationTarget::LanguageConstruct { span, .. } => *span,
        }
    }
}

impl HasSpan for InvocationArgumentsSource<'_> {
    fn span(&self) -> Span {
        match self {
            InvocationArgumentsSource::ArgumentList(arg_list) => arg_list.span(),
            InvocationArgumentsSource::PipeInput(pipe) => pipe.span(),
            InvocationArgumentsSource::Slice(expr_list) => {
                let first = expr_list.first();
                let last = expr_list.last();

                if let (Some(first), Some(last)) = (first, last) {
                    first.span().join(last.span())
                } else {
                    unreachable!("Expression list should have at least one element")
                }
            }
            InvocationArgumentsSource::None(span) => *span,
        }
    }
}

impl HasSpan for InvocationArgument<'_> {
    fn span(&self) -> Span {
        match self {
            InvocationArgument::Argument(arg) => arg.span(),
            InvocationArgument::PipedValue(expr) => expr.span(),
            InvocationArgument::Expression(expr) => expr.span(),
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::issue::TypingIssueKind;
    use crate::test_analysis;

    test_analysis! {
        name = type_narrowing_and_assertions,
        code = indoc! {r#"
            <?php

            /**
             * @assert-if-true array<array-key, mixed> $value
             *
             * @return ($value is array ? true : false)
             */
            function is_array(mixed $value): bool
            {
                return is_array($value);
            }

            /**
             * @template K as array-key
             * @template V
             *
             * @param array<K, V> $array
             *
             * @return list<V>
             */
            function array_values(array $array): array
            {
                return array_values($array);
            }

            /**
             * @template K as array-key
             * @template V
             * @template S
             * @template U
             *
             * @param (callable(V, S): U)|null $callback
             * @param array<K, V> $array
             * @param array<array-key, S> ...$arrays
             *
             * @return array<K, U>
             */
            function array_map(null|callable $callback, array $array, array ...$arrays): array
            {
                return array_map($callback, $array, ...$arrays);
            }

            /**
             * @template Tk
             * @template Tv
             * @template T
             *
             * @param iterable<Tk, Tv> $iterable Iterable to be mapped over
             * @param (Closure(Tv): T) $function
             *
             * @return ($iterable is non-empty-array ? non-empty-list<T> : list<T>)
             */
            function map(iterable $iterable, Closure $function): array
            {
                if (is_array($iterable)) {
                    return array_values(array_map($function, $iterable));
                }

                $result = [];
                foreach ($iterable as $value) {
                    $result[] = $function($value);
                }

                return $result;
            }
        "#}
    }

    test_analysis! {
        name = conditional_return_resolved_to_left,
        code = indoc! {r#"
            <?php

            /**
             * @param non-empty-string[] $strings
             *
             * @return ($strings is non-empty-array ? non-empty-string : string)
             */
            function join_strings(array $strings): string
            {
                $result = '';
                foreach ($strings as $string) {
                    $result .= $string;
                }

                return $result;
            }

            /**
             * @return non-empty-string
             */
            function x1(): string
            {
                return join_strings(['Hello', ' ', 'World!']);
            }

            /**
             * @return non-empty-string
             */
            function x2(): string
            {
                return join_strings(['a' => 'Hello', 'b' => ' ', 'c' => 'World!']);
            }
        "#}
    }

    test_analysis! {
        name = conditional_return_resolved_to_right,
        code = indoc! {r#"
            <?php

            /**
             * @param non-empty-string[] $strings
             *
             * @return ($strings is non-empty-array ? non-empty-string : string)
             */
            function join_strings(array $strings): string
            {
                $result = '';
                foreach ($strings as $string) {
                    $result .= $string;
                }

                return $result;
            }

            /**
             * @return non-empty-string
             */
            function x1(): string
            {
                return join_strings([]);
            }
        "#},
        issues = [
            TypingIssueKind::InvalidReturnStatement
        ]
    }

    test_analysis! {
        name = collection_types,
        code = indoc! {r#"
            <?php

            /**
             * @template K as array-key
             * @template V
             *
             * @param array<K, V> $array
             * @param V $filter_value
             * @param bool $strict
             *
             * @return ($array is non-empty-array ? non-empty-list<K> : list<K>)
             */
            function array_keys(array $array, mixed $filter_value = null, bool $strict = false): array
            {
                return array_keys($array, $filter_value, $strict);
            }

            /**
             * @template K as array-key
             * @template V
             *
             * @param array<K, V> $array
             *
             * @return ($array is non-empty-array ? non-empty-list<V> : list<V>)
             */
            function array_values(array $array): array
            {
                return array_values($array);
            }

            /**
             * @template Tk
             * @template Tv
             * @template T
             *
             * @param iterable<Tk, Tv> $iterable
             * @param (Closure(Tv): T) $function
             *
             * @return ($iterable is non-empty-array ? non-empty-list<T> : list<T>)
             */
            function vec_map(iterable $iterable, Closure $function): array
            {
                return vec_map($iterable, $function);
            }

            /**
             * @template Tv
             * @template Tu
             *
             * @param iterable<Tv> $first
             * @param iterable<Tu> $second
             *
             * @return list<array{0: Tv, 1: Tu}>
             */
            function vec_zip(iterable $first, iterable $second): array
            {
                return vec_zip($first, $second);
            }

            /**
             * @template T
             *
             * @param iterable<T> $iterable
             * @param positive-int $size
             *
             * @return list<list<T>>
             */
            function vec_chunk(iterable $iterable, int $size): array
            {
                return vec_chunk($iterable, $size);
            }

            /**
             * @template Tk of array-key
             * @template Tv
             */
            interface CollectionInterface
            {
                /**
                 * @return array<Tk, Tv>
                 */
                public function toArray(): array;

                /**
                 * @template Tu
                 *
                 * @param array<array-key, Tu> $elements
                 *
                 * @return CollectionInterface<Tk, array{0: Tv, 1: Tu}>
                 */
                public function zip(array $elements): CollectionInterface;

                /**
                 * @param positive-int $size
                 *
                 * @return CollectionInterface<int<0, max>, static<Tk, Tv>>
                 */
                public function chunk(int $size): CollectionInterface;
            }

            /**
             * @template Tk of array-key
             * @template Tv
             *
             * @extends CollectionInterface<Tk, Tv>
             */
            interface MutableCollectionInterface extends CollectionInterface
            {
                /**
                 * @template Tu
                 *
                 * @param array<array-key, Tu> $elements
                 *
                 * @return MutableCollectionInterface<Tk, array{0: Tv, 1: Tu}>
                 */
                public function zip(array $elements): MutableCollectionInterface;

                /**
                 * @param positive-int $size
                 *
                 * @return MutableCollectionInterface<int<0, max>, static<Tk, Tv>>
                 */
                public function chunk(int $size): MutableCollectionInterface;
            }

            /**
             * @template Tk of array-key
             * @template Tv
             *
             * @extends CollectionInterface<Tk, Tv>
             */
            interface AccessibleCollectionInterface extends CollectionInterface
            {
                /**
                 * @return AccessibleCollectionInterface<int<0, max>, Tv>
                 */
                public function values(): AccessibleCollectionInterface;

                /**
                 * @return AccessibleCollectionInterface<int<0, max>, Tk>
                 */
                public function keys(): AccessibleCollectionInterface;

                /**
                 * @template Tu
                 *
                 * @param array<array-key, Tu> $elements
                 *
                 * @return AccessibleCollectionInterface<Tk, array{0: Tv, 1: Tu}>
                 */
                public function zip(array $elements): AccessibleCollectionInterface;

                /**
                 * @param positive-int $size
                 *
                 * @return AccessibleCollectionInterface<int<0, max>, static<Tk, Tv>>
                 */
                public function chunk(int $size): AccessibleCollectionInterface;
            }

            /**
             * @template Tk of array-key
             * @template Tv
             *
             * @extends AccessibleCollectionInterface<Tk, Tv>
             * @extends MutableCollectionInterface<Tk, Tv>
             */
            interface MutableAccessibleCollectionInterface extends AccessibleCollectionInterface, MutableCollectionInterface
            {
                /**
                 * @return MutableAccessibleCollectionInterface<int<0, max>, Tv>
                 */
                public function values(): MutableAccessibleCollectionInterface;

                /**
                 * @return MutableAccessibleCollectionInterface<int<0, max>, Tk>
                 */
                public function keys(): MutableAccessibleCollectionInterface;

                /**
                 * @template Tu
                 *
                 * @param array<array-key, Tu> $elements
                 *
                 * @return MutableAccessibleCollectionInterface<Tk, array{0: Tv, 1: Tu}>
                 */
                public function zip(array $elements): MutableAccessibleCollectionInterface;

                /**
                 * @param positive-int $size
                 *
                 * @return MutableAccessibleCollectionInterface<int<0, max>, static<Tk, Tv>>
                 */
                public function chunk(int $size): MutableAccessibleCollectionInterface;
            }

            /**
             * @template T
             *
             * @extends AccessibleCollectionInterface<int<0, max>, T>
             */
            interface VectorInterface extends AccessibleCollectionInterface
            {
                /**
                 * @return list<T>
                 */
                public function toArray(): array;

                /**
                 * @return VectorInterface<T>
                 */
                public function values(): VectorInterface;

                /**
                 * @return VectorInterface<int<0, max>>
                 */
                public function keys(): VectorInterface;

                /**
                 * @template Tu
                 *
                 * @param array<array-key, Tu> $elements
                 *
                 * @return VectorInterface<array{0: T, 1: Tu}>
                 */
                public function zip(array $elements): VectorInterface;

                /**
                 * @param positive-int $size
                 *
                 * @return VectorInterface<static<T>>
                 */
                public function chunk(int $size): VectorInterface;
            }

            /**
             * @template T
             *
             * @extends VectorInterface<T>
             * @extends MutableAccessibleCollectionInterface<int<0, max>, T>
             */
            interface MutableVectorInterface extends MutableAccessibleCollectionInterface, VectorInterface
            {
                /**
                 * @return list<T>
                 */
                public function toArray(): array;

                /**
                 * @return MutableVectorInterface<T>
                 */
                public function values(): MutableVectorInterface;

                /**
                 * @return MutableVectorInterface<int<0, max>>
                 */
                public function keys(): MutableVectorInterface;

                /**
                 * @template Tu
                 *
                 * @param array<array-key, Tu> $elements
                 *
                 * @return MutableVectorInterface<array{0: T, 1: Tu}>
                 */
                public function zip(array $elements): MutableVectorInterface;

                /**
                 * @param positive-int $size
                 *
                 * @return MutableVectorInterface<static<T>>
                 */
                public function chunk(int $size): MutableVectorInterface;
            }

            /**
             * @template Tk of array-key
             * @template Tv
             *
             * @extends AccessibleCollectionInterface<Tk, Tv>
             */
            interface MapInterface extends AccessibleCollectionInterface
            {
                /**
                 * @return VectorInterface<Tv>
                 */
                public function values(): VectorInterface;

                /**
                 * @return VectorInterface<Tk>
                 */
                public function keys(): VectorInterface;

                /**
                 * @template Tu
                 *
                 * @param array<array-key, Tu> $elements
                 *
                 * @return MapInterface<Tk, array{0: Tv, 1: Tu}>
                 */
                public function zip(array $elements): MapInterface;

                /**
                 * @param positive-int $size
                 *
                 * @return VectorInterface<static<Tk, Tv>>
                 */
                public function chunk(int $size): VectorInterface;
            }

            /**
             * @template Tk of array-key
             * @template Tv
             *
             * @extends MapInterface<Tk, Tv>
             * @extends MutableAccessibleCollectionInterface<Tk, Tv>
             */
            interface MutableMapInterface extends MapInterface, MutableAccessibleCollectionInterface
            {
                /**
                 * @return MutableVectorInterface<Tv>
                 */
                public function values(): MutableVectorInterface;

                /**
                 * @return MutableVectorInterface<Tk>
                 */
                public function keys(): MutableVectorInterface;

                /**
                 * @template Tu
                 *
                 * @param array<array-key, Tu> $elements
                 *
                 * @return MutableMapInterface<Tk, array{0: Tv, 1: Tu}>
                 */
                public function zip(array $elements): MutableMapInterface;

                /**
                 * @param positive-int $size
                 *
                 * @return MutableVectorInterface<static<Tk, Tv>>
                 */
                public function chunk(int $size): MutableVectorInterface;
            }

            /**
             * @template T
             *
             * @implements MutableVectorInterface<T>
             */
            final class MutableVector implements MutableVectorInterface
            {
                /**
                 * @var list<T> $elements
                 */
                private array $elements = [];

                /**
                 * @param array<array-key, T> $elements
                 */
                public function __construct(array $elements)
                {
                    foreach ($elements as $element) {
                        $this->elements[] = $element;
                    }
                }

                /**
                 * @template Ts
                 *
                 * @param array<array-key, Ts> $elements
                 *
                 * @return MutableVector<Ts>
                 */
                public static function fromArray(array $elements): MutableVector
                {
                    return new self($elements);
                }

                /**
                 * @return list<T>
                 */
                public function toArray(): array
                {
                    return $this->elements;
                }

                /**
                 * @return MutableVector<T>
                 */
                public function values(): MutableVector
                {
                    return MutableVector::fromArray($this->elements);
                }

                /**
                 * @return MutableVector<int<0, max>>
                 */
                public function keys(): MutableVector
                {
                    return MutableVector::fromArray(array_keys($this->elements));
                }

                /**
                 * @template Tu
                 *
                 * @param array<array-key, Tu> $elements
                 *
                 * @return MutableVector<array{0: T, 1: Tu}>
                 */
                public function zip(array $elements): MutableVector
                {
                    return MutableVector::fromArray(vec_zip($this->elements, $elements));
                }

                /**
                 * @param positive-int $size
                 *
                 * @return MutableVector<MutableVector<T>>
                 */
                public function chunk(int $size): MutableVector
                {
                    return static::fromArray(vec_map(
                        vec_chunk($this->toArray(), $size),
                        /**
                         * @param list<T> $chunk
                         *
                         * @return MutableVector<T>
                         */
                        static fn(array $chunk): MutableVector => MutableVector::fromArray($chunk),
                    ));
                }
            }

            /**
             * @template Tk of array-key
             * @template Tv
             *
             * @implements MutableMapInterface<Tk, Tv>
             */
            final class MutableMap implements MutableMapInterface
            {
                /**
                 * @var array<Tk, Tv> $elements
                 */
                private array $elements;

                /**
                 * @param array<Tk, Tv> $elements
                 */
                public function __construct(array $elements)
                {
                    $this->elements = $elements;
                }

                /**
                 * @template Tsk of array-key
                 * @template Tsv
                 *
                 * @param array<Tsk, Tsv> $elements
                 *
                 * @return MutableMap<Tsk, Tsv>
                 */
                public static function fromArray(array $elements): MutableMap
                {
                    return new self($elements);
                }

                /**
                 * @return array<Tk, Tv>
                 */
                public function toArray(): array
                {
                    return $this->elements;
                }

                /**
                 * @return MutableVector<Tv>
                 */
                public function values(): MutableVector
                {
                    return MutableVector::fromArray($this->elements);
                }

                /**
                 * @return MutableVector<Tk>
                 */
                public function keys(): MutableVector
                {
                    return MutableVector::fromArray(array_keys($this->elements));
                }

                /**
                 * @template Tu
                 *
                 * @param array<array-key, Tu> $elements
                 *
                 * @return MutableMap<Tk, array{0: Tv, 1: Tu}>
                 */
                public function zip(array $elements): MutableMap
                {
                    return $this->zip($elements); // stub
                }

                /**
                 * @param positive-int $size
                 *
                 * @return MutableVector<MutableMap<Tk, Tv>>
                 */
                public function chunk(int $size): MutableVector
                {
                    $chunks = $this->zip($this->keys()->toArray())
                        ->values()
                        ->chunk($size)
                        ->toArray();

                    return MutableVector::fromArray(vec_map(
                        $chunks,
                        /**
                         * @param MutableVector<array{0: Tv, 1: Tk}> $vector
                         *
                         * @return MutableMap<Tk, Tv>
                         */
                        static function (MutableVector $vector): MutableMap {
                            /** @var array<Tk, Tv> $array */
                            $array = [];
                            foreach ($vector->toArray() as [$v, $k]) {
                                $array[$k] = $v;
                            }

                            return MutableMap::fromArray($array);
                        },
                    ));
                }
            }
        "#},
    }

    test_analysis! {
        name = recursive_templates,
        code = indoc! {r#"
            <?php


            /**
             * @template K as array-key
             * @template V
             *
             * @param array<K, V> $array
             * @param V $filter_value
             * @param bool $strict
             *
             * @return ($array is non-empty-array ? non-empty-list<K> : list<K>)
             */
            function array_keys(array $array, mixed $filter_value = null, bool $strict = false): array {
                return array_keys($array, $filter_value, $strict);
            }

            /**
             * @template K as array-key
             * @template V
             *
             * @param array<K, V> $array
             *
             * @return ($array is non-empty-array ? non-empty-list<V> : list<V>)
             */
            function array_values(array $array): array {
                return array_values($array);
            }

            /**
             * @template Tk of array-key
             * @template Tv
             */
            final class Map
            {
                /**
                 * @var array<Tk, Tv> $elements
                 */
                private array $elements;

                /**
                 * @param array<Tk, Tv> $elements
                 */
                public function __construct(array $elements = [])
                {
                    $this->elements = $elements;
                }

                /**
                 * @template Tu
                 *
                 * @param array<array-key, Tu> $elements
                 *
                 * @return Map<Tk, array{0: Tv, 1: Tu}>
                 */
                public function zip(array $elements): Map
                {
                    return $this->zip($elements);
                }

                /**
                 * @return list<Tk>
                 */
                public function keys(): array
                {
                    return array_keys($this->elements);
                }

                /**
                 * @return list<Tv>
                 */
                public function values(): array
                {
                    return array_values($this->elements);
                }

                /**
                 * @return array{
                 *   keys: list<Tk>,
                 *   values: list<Tv>,
                 *   zipped_with_keys: Map<Tk, array{0: Tv, 1: Tk}>,
                 *   zipped_with_values: Map<Tk, array{0: Tv, 1: Tv}>,
                 *   values_of_zipped_with_keys: list<array{0: Tv, 1: Tk}>,
                 *   values_of_zipped_with_values: list<array{0: Tv, 1: Tv}>
                 * }
                 */
                public function test(): array
                {
                    $keys = $this->keys();
                    $values = $this->values();
                    $zipped_with_keys = $this->zip($keys);
                    $zipped_with_values = $this->zip($values);
                    $values_of_zipped_with_keys = $zipped_with_keys->values();
                    $values_of_zipped_with_values = $zipped_with_values->values();

                    return [
                        'keys' => $keys,
                        'values' => $values,
                        'zipped_with_keys' => $zipped_with_keys,
                        'zipped_with_values' => $zipped_with_values,
                        'values_of_zipped_with_keys' => $values_of_zipped_with_keys,
                        'values_of_zipped_with_values' => $values_of_zipped_with_values,
                    ];
                }
            }
        "#},
    }

    test_analysis! {
        name = psl_integration,
        code = indoc! {r#"
            <?php

            declare(strict_types=1);

            namespace Psl\Type {
                /**
                 * @template T
                 */
                interface TypeInterface
                {
                    /**
                     * @param mixed $value
                     *
                     * @return T
                     */
                    public function assert($value): mixed;
                }

                /**
                 * @template Tk of array-key
                 * @template Tv
                 *
                 * @param array<Tk, TypeInterface<Tv>> $elements
                 *
                 * @return TypeInterface<array<Tk, Tv>>
                 */
                function shape(array $elements, bool $allow_unknown_fields = false): TypeInterface
                {
                    return shape($elements, $allow_unknown_fields);
                }

                /**
                 * @return TypeInterface<string>
                 */
                function string(): TypeInterface
                {
                    return string();
                }

                /**
                 * @return TypeInterface<int>
                 */
                function int(): TypeInterface
                {
                    return int();
                }

                /**
                 * @template T
                 * @param class-string<T> $class_name
                 * @return TypeInterface<T>
                 */
                function instance_of(string $class_name): TypeInterface
                {
                    return instance_of($class_name);
                }
            }

            namespace {
                enum Example
                {
                    case Foo;
                    case Bar;
                }

                function get_mixed(): mixed
                {
                    return 1;
                }

                function i_take_string(string $value): void
                {
                    echo "Received string: $value\n";
                }

                function i_take_int(int $value): void
                {
                    echo "Received int: $value\n";
                }

                function i_take_enum(Example $value): void
                {
                    echo
                        match ($value) {
                            Example::Foo => "Received enum: Foo\n",
                            Example::Bar => "Received enum: Bar\n",
                        }
                    ;
                }

                $array_type = Psl\Type\shape([
                    'name' => Psl\Type\string(),
                    'age' => Psl\Type\int(),
                    'address' => Psl\Type\shape([
                        'street' => Psl\Type\string(),
                        'city' => Psl\Type\string(),
                    ]),
                ]);

                $list_type = Psl\Type\shape([
                    Psl\Type\string(),
                    Psl\Type\int(),
                    Psl\Type\shape([
                        'street' => Psl\Type\string(),
                        'city' => Psl\Type\string(),
                    ]),
                ]);

                $enum_type = Psl\Type\instance_of(Example::class);

                $array = $array_type->assert(get_mixed());
                $list = $list_type->assert(get_mixed());
                $enum = $enum_type->assert(get_mixed());

                i_take_string($array['name']);
                i_take_int($array['age']);
                i_take_string($array['address']['street']);
                i_take_string($array['address']['city']);

                i_take_string($list[0]);
                i_take_int($list[1]);
                i_take_string($list[2]['street']);
                i_take_string($list[2]['city']);

                i_take_enum($enum);
            }
        "#},
    }

    test_analysis! {
        name = argument_count,
        code = indoc! {r#"
            <?php

            final readonly class Number {
                public function __construct(
                    private int $value,
                ) {}

                public function getValue(): int {
                    return $this->value;
                }
            }

            final readonly class Calculator {
                public static function sum(Number $first, Number ...$rest): Number {
                    $total = $first->getValue();
                    foreach ($rest as $number) {
                        $total += $number->getValue();
                    }

                    return new Number($total);
                }
            }

            function sum_numbers(Number ...$number): Number {
                return $number === [] ? new Number(0) : Calculator::sum(...$number);
            }

            $a = new Number(10);
            $b = new Number(20);
            $c = new Number(30);

            $_result = Calculator::sum(...[$a, $b, $c]);
            $_result = Calculator::sum($a, ...[$b, $c]);
            $_result = Calculator::sum($a, $b, ...[$c]);
            $_result = Calculator::sum($a, $b, $c);
        "#},
    }

    test_analysis! {
        name = unspecified_callable_or_closure,
        code = indoc! {r#"
            <?php

            /** @param Closure $callback */
            function configureScope(Closure $callback): mixed {
                return $callback(1, 2, 3);
            }

            configureScope(function (string $x): string {
                return 'A' . $x;
            });

            /** @param callable $callback */
            function configureScopeWithCallable(callable $callback): mixed {
                return $callback(1, 2, 3);
            }

            configureScopeWithCallable(function (string $x): string {
                return 'A' . $x;
            });
        "#},
    }

    test_analysis! {
        name = untyped_callable_parameter,
        code = indoc! {r#"
            <?php

            /**
             * @param callable(...):void $callable
             */
            function queue(callable $callable): void
            {
                $callable();
            }

            queue(function (): void {});
            queue(function (string $x): string {
                return $x;
            });
            queue(function (string $x, string $y): string {
                return $x . $y;
            });
        "#},
    }
}
