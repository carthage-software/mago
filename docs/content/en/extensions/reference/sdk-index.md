+++
title = "PHP SDK index"
description = "A categorized index of the public extension authoring API."
nav_order = 10
nav_section = "Extensions"
nav_subsection = "Reference"
+++
# PHP SDK index

This page indexes the public extension authoring API. Public symbols live under `Mago\Sdk` and carry `@api`. Symbols under `Mago\Sdk\Internal` implement the bundled binary protocol and are not a compatibility surface.

## Runtime

| Type | Purpose |
| :--- | :--- |
| `Extension` | Declares one logical extension's rules, plugins, version, and optional reducer |
| `Worker` | Hosts one or more extensions in one command-scoped worker process |
| `WorkerReducer` | Exports and merges process-local state at pool shutdown |
| `WorkerReductionContext` | Ordered worker payloads and cancellation for the surviving reducer |
| `CancellationTokenInterface` | Cooperative request cancellation |
| `PHPVersion` | Mago's configured target language version |
| `Span` | Half-open byte range within a source file |
| `SourceLocation` | Logical filename plus span |

See [Extensions and workers](/extensions/sdk/extensions-and-workers/) and [Worker state and reduction](/extensions/sdk/worker-reduction/).

## Exceptions

`SdkException` is the common marker. `InvalidArgumentException` reports invalid public DTO construction, `ProtocolException` reports a worker/protocol contract failure, and `CancelledException` terminates cancelled callback work.

## Syntax

| Type | Purpose |
| :--- | :--- |
| `Syntax\SourceFile` | Immutable capability-specific source, syntax, names, optional literals, and optional comment trivia |
| `Syntax\Node` | One concrete-syntax node |
| `Syntax\NodeKind` | Generated syntax-node enum used by protocol snapshots |
| `Syntax\ResolvedName` | Resolved semantic name and source span |
| `Syntax\Trivia`, `Syntax\TriviaKind` | Comments and their lexical kind |
| `Syntax\CallExpression` | Structured function or method call view |
| `Syntax\CallArgument` | Structured call argument view |

See [Syntax nodes and source files](/extensions/sdk/syntax-and-source/).

## Reporting

| Type | Purpose |
| :--- | :--- |
| `Reporting\Issue` | Immutable extension diagnostic builder |
| `Reporting\ReportedIssue` | Read-only issue with effective code and level |
| `Reporting\Annotation`, `Reporting\AnnotationKind` | Primary and secondary source annotations |
| `Reporting\Level` | Note, help, warning, or error severity |
| `Reporting\TextEdit` | Delete, insert, or replace suggestion |
| `Reporting\Safety` | Safe, potentially unsafe, or unsafe edit classification |

See [Reporting issues and suggested edits](/extensions/sdk/reporting/).

## Linter

| Type | Purpose |
| :--- | :--- |
| `Linter\Rule` | Custom linter rule contract |
| `Linter\RuleDefinition` | Code, name, description, level, activation, and syntax targets |
| `Linter\LintContext` | Current source, target node, traversal, names, cancellation, and reporting |

See [Writing linter rules](/extensions/linter/rules/).

## Analyzer plugin registration

| Type | Purpose |
| :--- | :--- |
| `Analyzer\Plugin` | Analyzer plugin contract |
| `Analyzer\PluginDefinition` | Identifier, aliases, description, and default activation |
| `Analyzer\PluginRegistry` | Registers every provider, hook, entry point, filter, and memoization policy |
| `Analyzer\TargetedProvider` | Common non-empty target-list contract |
| `Analyzer\TargetedAnalysisHook` | Common targets, requirements, and post-analysis node callback |

See [Analyzer plugins](/extensions/analyzer/overview/).

## Analyzer lifecycle

| Hook | Context and data |
| :--- | :--- |
| `InitializationHook` | `InitializationContext`: PHP version, cancellation, in-memory stubs |
| `CodebaseScanHook` | `CodebaseScanContext`: deterministic batches of selected host source |
| `BeforeAnalysisHook` | `BeforeAnalysisContext`: frozen codebase and project-wide reference registry |
| `AfterFileAnalysisHook` | `AfterFileAnalysisContext`: completed `FileAnalysis` and file references |
| `AfterAnalysisHook` | `AfterAnalysisContext`: merged `ProjectAnalysis` and final reference graph |

`LifecycleContext` supplies PHP version, `Codebase`, `TypeComparator`, cancellation, and analyzer issue reporting. `FileAnalysisRequirement::ExpressionTypes` configures plain after-file hooks; the remaining cases configure targeted hooks.

See [Lifecycle hooks](/extensions/analyzer/lifecycle-hooks/).

## Return types and call signatures

| Type | Purpose |
| :--- | :--- |
| `FunctionReturnTypeProvider` | Refines selected function return types |
| `MethodReturnTypeProvider` | Refines selected instance or static method return types |
| `ReturnTypeProviderContext` | Invocation with inferred argument types and semantic services |
| `CallableSignatureProvider` | Establishes parameters for an unresolved callable |
| `CallableSignatureOverride` | Marks a signature provider that may replace a declaration |
| `UndeclaredReturnTypeProvider` | Marks a return provider as unresolved-callable-only |
| `CallableSignatureProviderContext` | Pre-argument invocation and semantic services |
| `EffectiveCallableSignature` | Parameters and named-argument policy used by Mago |
| `Type\CallableParameter` | Parameter name, type, closure `$this`, reference, variadic, and default facts |
| `Invocation`, `InvocationKind` | Function, instance-method, or static-method call |
| `Argument` | Named/positional source argument and optional inferred type |

Targets are `FunctionTarget` with `FunctionTargetKind`, or `MethodTarget`. See [Return types and callable signatures](/extensions/analyzer/return-types-and-callable-signatures/).

## Assertions

`FunctionAssertionProvider`, `MethodAssertionProvider`, and `AssertionProviderContext` produce `InvocationAssertions`.

The assertion value families are:

- `Assertion\TypeAssertion` and `TypeAssertionKind`;
- `Assertion\SimpleAssertion` and `SimpleAssertionKind`;
- `Assertion\IntegerAssertion` and `IntegerAssertionKind`;
- `Assertion\ArrayKeyAssertion` and `ArrayKeyAssertionKind`;
- `Assertion\CountabilityAssertion` and `CountabilityAssertionKind`;
- `Assertion\VariableAssertion` and `VariableAssertionKind`.

All implement the `Assertion\Assertion` marker. See [Assertion providers](/extensions/analyzer/assertion-providers/).

## Properties and initialization

| Type | Purpose |
| :--- | :--- |
| `PropertyTypeProvider`, `PropertyTypeProviderContext` | Establish a dynamic property or override a selected property's access types |
| `PropertyType` | Optional read and write contracts |
| `PropertyAccess`, `PropertyAccessKind` | Current property, receiver, span, and read/write operation |
| `PropertyInitializationProvider`, `PropertyInitializationProviderContext` | Mark a selected declared property initialized |
| `ClassInitializerProvider`, `ClassInitializerProviderContext` | Declare lifecycle methods that initialize properties |

Targets are `PropertyTarget` and `ClassTarget`. See [Properties and initialization](/extensions/analyzer/properties-and-initialization/).

## Targeted analysis

| Hook | Target |
| :--- | :--- |
| `NodeAnalysisHook` | Exact `Syntax\NodeKind` cases |
| `MethodCallAnalysisHook` | Resolved `MethodTarget` patterns |
| `ClassLikeAnalysisHook` | `ClassLikeTarget` descendant declarations |

All receive `NodeAnalysisContext`. See [Targeted analysis hooks](/extensions/analyzer/targeted-analysis-hooks/).

## Entry points and filters

`AttributedEntryPoint`, `ClassTarget`, and `MethodTarget` describe native framework entry points. `IssueFilterHook`, `IssueFilterContext`, and `IssueFilterDecision` implement targeted last-resort filtering. See [Entry points and issue filtering](/extensions/analyzer/entry-points-and-issue-filtering/).

## Analysis results and references

| Type | Purpose |
| :--- | :--- |
| `FileAnalysis` | Completed per-file summary and lazy artifacts |
| `ExpressionType` | Expression span plus inferred type |
| `ProjectAnalysis` | Final files, issue count, and merged references |
| `ReferenceRegistry` | Contributes framework-known edges |
| `ReferenceOrigin` | Symbol/member or file edge origin |
| `ReferenceKind` | Body, signature, overridden-member, function-like-return, property-read, or property-write edge |
| `ReferenceSummary` | Aggregate reference counts |
| `SymbolReferences` | Lazy final graph queries |
| `SymbolReference` | One directed graph edge |

See [Analysis results and references](/extensions/analyzer/analysis-results-and-references/).

## Codebase metadata

`Codebase` queries immutable Mago metadata. Public metadata DTOs are:

- `Metadata\ClassLikeMetadata` and `ClassLikeKind`;
- `Metadata\FunctionLikeMetadata` and `FunctionLikeKind`;
- `Metadata\ParameterMetadata`;
- `Metadata\PropertyMetadata` and `PropertyHookMetadata`;
- `Metadata\ClassConstantMetadata`, `EnumCaseMetadata`, and `ConstantMetadata`;
- `Metadata\TemplateMetadata` and `TypeMetadata`;
- `Metadata\AttributeMetadata` and `AttributeArgumentMetadata`;
- `Metadata\MemberIdentifier`;
- `Metadata\MethodMetadataProjection` and `MethodFields`;
- `Metadata\MetadataFlags` and `VersionRange`.

See [Codebase metadata](/extensions/analyzer/codebase-metadata/).

## Types

`Analyzer\Type` is an immutable union. `TypeComparator`, `TypeComparison`, and `TypeComparisonKind` delegate equality, containment, and overlap to Mago.

Atomic values implement `Type\AtomicType`:

- `AliasType`, `ReferenceType`, and `VariableType`;
- `MixedType`, `SimpleAtomicType`, `ScalarType`, and `ResourceType`;
- `AnyObjectType`, `NamedObjectType`, and `EnumType`;
- `ObjectShapeType`, `ObjectWithMethodType`, and `ObjectWithPropertyType`;
- `KeyedArrayType`, `ListType`, and `IterableType`;
- `CallableType`, `GenericParameterType`, `ConditionalType`, and `DerivedType`.

Supporting type values and enums are:

- `ArrayItem`, `ArrayKey`, and `ArrayKeyKind`;
- `CallableConstraint`, `CallableParameter`, and `CallableSignature`;
- `ClassLikeStringType`, `ClassLikeStringKind`, and `ClassLikeStringVariant`;
- `FloatType`, `FloatTypeKind`, `IntegerType`, and `IntegerTypeKind`;
- `ScalarTypeKind`, `SimpleAtomicTypeKind`, and `MixedTruthiness`;
- `StringType`, `StringLiteralKind`, and `StringCasing`;
- `FunctionLikeIdentifier` and `Type\FunctionLikeKind`;
- `GenericParent`, `GenericParentKind`, `Variance`, and `Visibility`;
- `ListElement`, `ObjectProperty`, and `TypeFlags`;
- `DerivedTypeKind`, `ReferenceTypeKind`, and `ReferenceSelectorKind`.

See [Types and comparisons](/extensions/analyzer/types-and-comparisons/).
