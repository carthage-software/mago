#![allow(clippy::too_many_arguments)]

use mago_codex::context::ScopeContext;
use mago_codex::data_flow::graph::DataFlowGraph;
use mago_codex::metadata::CodebaseMetadata;
use mago_interner::ThreadedInterner;
use mago_names::ResolvedNames;
use mago_source::Source;
use mago_span::HasSpan;
use mago_syntax::ast::Program;

use crate::analysis_result::AnalysisResult;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::settings::Settings;
use crate::statement::analyze_statements;

pub mod analysis_result;
pub mod error;
pub mod settings;

mod analyzable;
mod artifacts;
mod assertion;
mod context;
mod dataflow;
mod expression;
mod formula;
mod invocation;
mod issue;
mod reconciler;
mod resolver;
mod statement;
mod utils;
mod visibility;

#[derive(Clone, Debug)]
pub struct Analyzer<'a> {
    pub source: Source,
    pub resolved_names: &'a ResolvedNames,
    pub codebase: &'a CodebaseMetadata,
    pub interner: &'a ThreadedInterner,
    pub settings: Settings,
}

impl<'a> Analyzer<'a> {
    pub fn new(
        source: Source,
        resolved_names: &'a ResolvedNames,
        codebase: &'a CodebaseMetadata,
        interner: &'a ThreadedInterner,
        settings: Settings,
    ) -> Self {
        Self { source, resolved_names, codebase, interner, settings }
    }

    pub fn analyze(&mut self, program: &Program, analysis_result: &mut AnalysisResult) -> Result<(), AnalysisError> {
        let start_time = std::time::Instant::now();

        if !program.has_script() {
            analysis_result.time_in_analysis = start_time.elapsed();
            return Ok(());
        }

        let statements = program.statements.as_slice();

        let mut context = {
            Context::new(
                self.interner,
                self.codebase,
                &self.source,
                self.resolved_names,
                &self.settings,
                statements[0].span(),
                program.trivia.as_slice(),
            )
        };

        let mut block_context = BlockContext::new(ScopeContext::new());
        let mut artifacts = AnalysisArtifacts::new(DataFlowGraph::new(self.settings.graph_kind));

        analyze_statements(statements, &mut context, &mut block_context, &mut artifacts)?;

        context.finish(artifacts, analysis_result, false);

        analysis_result.time_in_analysis = start_time.elapsed();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use ahash::HashSet;
    use indoc::indoc;

    use mago_codex::metadata::CodebaseMetadata;
    use mago_codex::populator::populate_codebase;
    use mago_codex::reference::SymbolReferences;
    use mago_codex::scanner::scan_program;
    use mago_interner::ThreadedInterner;
    use mago_names::resolver::NameResolver;
    use mago_reporting::Issue;
    use mago_source::Source;
    use mago_syntax::parser::parse_source;

    use crate::Analyzer;
    use crate::analysis_result::AnalysisResult;
    use crate::settings::Settings;

    #[derive(Debug, Clone)]
    pub struct TestCase<'a> {
        name: &'a str,
        content: &'a str,
        settings: Settings,
        expected_issues: Vec<String>,
    }

    impl<'a> TestCase<'a> {
        pub fn new(name: &'a str, content: &'a str) -> Self {
            Self { name, content, settings: Settings::default(), expected_issues: vec![] }
        }

        pub fn expect_success(mut self) -> Self {
            self.expected_issues = vec![];
            self
        }

        pub fn expect_issues(mut self, kinds: Vec<String>) -> Self {
            self.expected_issues = kinds;
            self
        }

        pub fn run(self) {
            run_test_case_inner(self);
        }
    }

    fn run_test_case_inner(config: TestCase) {
        let interner = ThreadedInterner::new();
        let source = Source::standalone(&interner, config.name, config.content);

        let (program, parse_issues) = parse_source(&interner, &source);
        if parse_issues.is_some() {
            panic!("Test '{}' failed during parsing:\n{:#?}", config.name, parse_issues);
        }

        let resolver = NameResolver::new(&interner);
        let resolved_names = resolver.resolve(&program);
        let mut codebase = scan_program(&interner, &source, &program, &resolved_names);
        let mut symbol_references = SymbolReferences::new();

        populate_codebase(&mut codebase, &interner, &mut symbol_references, HashSet::default(), HashSet::default());

        let mut analysis_result = AnalysisResult::new(config.settings.graph_kind, symbol_references);
        let mut analyzer = Analyzer::new(source, &resolved_names, &codebase, &interner, config.settings);

        let analysis_run_result = analyzer.analyze(&program, &mut analysis_result);

        if let Err(err) = analysis_run_result {
            panic!("Test '{}': Expected analysis to succeed, but it failed with an error: {}", config.name, err);
        }

        verify_reported_issues(config.name, &analysis_result, codebase, &config.expected_issues);
    }

    fn verify_reported_issues(
        test_name: &str,
        analysis_result: &AnalysisResult,
        mut codebase: CodebaseMetadata,
        expected_issue_kinds: &[String],
    ) {
        let mut actual_issues_collected: Vec<Issue> = Vec::new();
        for issues_in_file in analysis_result.emitted_issues.values() {
            actual_issues_collected.extend(issues_in_file.clone());
        }

        actual_issues_collected.extend(codebase.take_issues(true));

        let actual_issues_count = actual_issues_collected.len();
        let mut expected_issue_counts: BTreeMap<String, usize> = BTreeMap::new();
        for kind in expected_issue_kinds {
            *expected_issue_counts.entry(kind.to_string()).or_insert(0) += 1;
        }

        let mut actual_issue_counts: BTreeMap<String, usize> = BTreeMap::new();
        for actual_issue in &actual_issues_collected {
            let Some(issue_code) = actual_issue.code.as_ref().cloned() else {
                panic!("Analyzer returned an issue with no code: {actual_issue:?}");
            };

            *actual_issue_counts.entry(issue_code).or_insert(0) += 1;
        }

        let mut discrepancies = Vec::new();

        for (actual_kind, &actual_count) in &actual_issue_counts {
            let expected_count = expected_issue_counts.get(actual_kind).copied().unwrap_or(0);
            if actual_count > expected_count {
                discrepancies.push(format!(
                    "- Unexpected issue(s) of kind `{}`: found {}, expected {}.",
                    actual_kind.as_str(),
                    actual_count,
                    expected_count
                ));
            }
        }

        for (expected_kind, &expected_count) in &expected_issue_counts {
            let actual_count = actual_issue_counts.get(expected_kind).copied().unwrap_or(0);
            if actual_count < expected_count {
                discrepancies.push(format!(
                    "- Missing expected issue(s) of kind `{}`: expected {}, found {}.",
                    expected_kind.as_str(),
                    expected_count,
                    actual_count
                ));
            }
        }

        if !discrepancies.is_empty() {
            let mut panic_message = format!("Test '{test_name}' failed with issue discrepancies:\n");
            for d in discrepancies {
                panic_message.push_str(&format!("  {d}\n"));
            }

            panic!("{}", panic_message);
        }

        if expected_issue_kinds.is_empty() && actual_issues_count != 0 {
            let mut panic_message = format!("Test '{test_name}': Expected no issues, but found:\n");
            for issue in actual_issues_collected {
                panic_message.push_str(&format!(
                    "  - Code: `{}`, Message: \"{}\"\n",
                    issue.code.unwrap_or_default(),
                    issue.message
                ));
            }

            panic!("{}", panic_message);
        }
    }

    #[macro_export]
    macro_rules! test_analysis {
        (name = $test_name:ident, code = $code_str:expr $(,)?) => {
            #[test]
            pub fn $test_name() {
                $crate::tests::TestCase::new(stringify!($test_name), $code_str).expect_success().run();
            }
        };
        (name = $test_name:ident, settings = $settings:expr, code = $code_str:expr $(,)?) => {
            #[test]
            pub fn $test_name() {
                $crate::tests::TestCase::new(stringify!($test_name), $code_str).settings($settings).expect_success().run();
            }
        };
        (name = $test_name:ident, code = $code_str:expr, issues = [$($issue_kind:expr),* $(,)?] $(,)?) => {
            #[test]
            pub fn $test_name() {
                $crate::tests::TestCase::new(stringify!($test_name), $code_str)
                    .expect_issues(vec![$($issue_kind.to_string()),*])
                    .run();
            }
        };
        (name = $test_name:ident, settings = $settings:expr, code = $code_str:expr, issues = [$($issue_kind:expr),* $(,)?] $(,)?) => {
            #[test]
            pub fn $test_name() {
                $crate::tests::TestCase::new(stringify!($test_name), $code_str)
                    .settings($settings)
                    .expect_issues(vec![$($issue_kind.to_string()),*])
                    .run();
            }
        };
    }

    test_analysis! {
        name = counting_iterable,
        code = indoc! {r#"
            <?php

            namespace {
                /**
                 * @template K
                 * @template V
                 */
                interface Traversable
                {
                }

                interface Countable
                {
                }

                function is_countable(mixed $_value): bool
                {
                    return false;
                }

                /**
                 * @return int<0, max>
                 */
                function count(mixed $_value): int
                {
                    return 0;
                }
            }

            namespace Example {
                use function Core\count;
                use function Core\is_countable;

                /**
                 * @template T
                 *
                 * @param iterable<T> $iterable
                 *
                 * @return int<0, max>
                 */
                function count_elements(iterable $iterable): int
                {
                    if (is_countable($iterable)) {
                        return count($iterable);
                    }

                    $count = 0;
                    foreach ($iterable as $_) {
                        ++$count;
                    }

                    return $count;
                }
            }
        "#},
    }

    test_analysis! {
        name = queue,
        code = indoc! {r#"
            <?php

            /**
             * Returns the largest element of the given list, or null if the
             * list is empty.
             *
             * @template T of int|float
             *
             * @param list<T> $numbers
             *
             * @return ($numbers is non-empty-list<T> ? T : null)
             *
             * @pure
             */
            function max_value(array $numbers): null|int|float
            {
                return max_value($numbers);
            }

            /**
             * @template K of array-key
             * @template V
             *
             * @param array<K, V> $array
             * @param-out ($array is list ? list<V> : array<K, V>) $array
             *
             * @return V|null
             *
             * @pure
             */
            function array_shift(array &$array): mixed
            {
                return array_shift($array);
            }

            /**
             * @template K as array-key
             * @template V
             *
             * @param array<K, V> $array
             * @param V $filter_value
             * @param bool $strict
             *
             * @return list<K>
             *
             * @no-named-arguments
             * @pure
             */
            function array_keys(array $array, mixed $filter_value = null, bool $strict = false): array
            {
                return array_keys($array, $filter_value, $strict);
            }

            /**
             * @return int<0, max>
             *
             * @pure
             */
            function count(Countable|array $value): int
            {
                return count($value);
            }

            #[Attribute(Attribute::TARGET_METHOD)]
            final class Override
            {
                public function __construct() {}
            }

            #[Attribute(Attribute::TARGET_CLASS)]
            final class Attribute
            {
                public const TARGET_CLASS = 1;
                public const TARGET_FUNCTION = 2;
                public const TARGET_METHOD = 4;
                public const TARGET_PROPERTY = 8;
                public const TARGET_CLASS_CONSTANT = 16;
                public const TARGET_PARAMETER = 32;
                public const TARGET_ALL = 63;
                public const IS_REPEATABLE = 64;

                public int $flags;

                public function __construct(int $flags = self::TARGET_ALL) {}
            }

            interface Stringable
            {
                public function __toString(): string;
            }

            interface Throwable extends Stringable
            {
                public function getMessage(): string;

                /**
                 * @return int|string
                 */
                public function getCode();

                public function getFile(): string;

                public function getLine(): int;

                public function getTrace(): array;

                public function getTraceAsString(): string;

                public function getPrevious(): Throwable|null;

                /**
                 * @return string
                 */
                public function __toString();
            }

            class Exception implements Throwable
            {
                protected $message;

                protected $code;

                protected string $file;

                protected int $line;

                /**
                 * @pure
                 */
                public function __construct(string $message = '', int $code = 0, null|Throwable $previous = null) {}

                /**
                 * @mutation-free
                 */
                final public function getMessage(): string
                {
                }

                /**
                 * @return int|string
                 *
                 * @mutation-free
                 */
                final public function getCode()
                {
                }

                /**
                 * @mutation-free
                 */
                final public function getFile(): string
                {
                }

                /**
                 * @mutation-free
                 */
                final public function getLine(): int
                {
                }

                /**
                 * @mutation-free
                 */
                final public function getTrace(): array
                {
                }

                /**
                 * @mutation-free
                 */
                final public function getPrevious(): null|Throwable
                {
                }

                /**
                 * @mutation-free
                 */
                final public function getTraceAsString(): string
                {
                }

                public function __toString(): string
                {
                }

                private function __clone(): void
                {
                }

                public function __wakeup(): void
                {
                }
            }

            class RuntimeException extends Exception
            {
            }

            class UnderflowException extends RuntimeException
            {
            }

            interface Countable
            {
                public function count(): int;
            }

            /**
             * An interface representing a queue data structure ( FIFO ).
             *
             * @template T
             */
            interface QueueInterface extends Countable
            {
                /**
                 * Adds a node to the queue.
                 *
                 * @param T $node
                 */
                public function enqueue(mixed $node): void;

                /**
                 * Retrieves, but does not remove, the node at the head of this queue,
                 * or returns null if this queue is empty.
                 *
                 * @return null|T
                 */
                public function peek(): mixed;

                /**
                 * Retrieves and removes the node at the head of this queue,
                 * or returns null if this queue is empty.
                 *
                 * @return null|T
                 */
                public function pull(): mixed;

                /**
                 * Retrieves and removes the node at the head of this queue.
                 *
                 * @return T
                 */
                public function dequeue(): mixed;

                /**
                 * Count the nodes in the queue.
                 *
                 * @return int<0, max>
                 */
                #[Override]
                public function count(): int;
            }

            /**
             * @template T
             *
             * @extends QueueInterface<T>
             */
            interface PriorityQueueInterface extends QueueInterface
            {
                /**
                 * Adds a node to the queue.
                 *
                 * @param T $node
                 */
                #[Override]
                public function enqueue(mixed $node, int $priority = 0): void;
            }

            /**
             * @template T
             *
             * @implements PriorityQueueInterface<T>
             */
            final class PriorityQueue implements PriorityQueueInterface
            {
                /**
                 * @var array<int, non-empty-list<T>>
                 */
                private array $queue = [];

                /**
                 * Adds a node to the queue.
                 *
                 * @param T $node
                 *
                 * @psalm-external-mutation-free
                 */
                #[Override]
                public function enqueue(mixed $node, int $priority = 0): void
                {
                    $nodes = $this->queue[$priority] ?? [];
                    $nodes[] = $node;

                    $this->queue[$priority] = $nodes;
                }

                /**
                 * Retrieves, but does not remove, the node at the head of this queue,
                 * or returns null if this queue is empty.
                 *
                 * @return null|T
                 *
                 * @psalm-mutation-free
                 */
                #[Override]
                public function peek(): mixed
                {
                    if (0 === $this->count()) {
                        return null;
                    }

                    $keys = array_keys($this->queue);

                    // Retrieve the highest priority.
                    $priority = max_value($keys) ?? 0;

                    // Retrieve the list of nodes with the priority `$priority`.
                    $nodes = $this->queue[$priority] ?? [];

                    // Retrieve the first node of the list.
                    return $nodes[0] ?? null;
                }

                /**
                 * Retrieves and removes the node at the head of this queue,
                 * or returns null if this queue is empty.
                 *
                 * @return null|T
                 *
                 * @psalm-external-mutation-free
                 */
                #[Override]
                public function pull(): mixed
                {
                    try {
                        return $this->dequeue();
                    } catch (UnderflowException) {
                        return null;
                    }
                }

                /**
                 * Dequeues a node from the queue.
                 *
                 * @throws UnderflowException If the queue is empty.
                 *
                 * @return T
                 *
                 * @psalm-external-mutation-free
                 */
                #[Override]
                public function dequeue(): mixed
                {
                    if (0 === $this->count()) {
                        throw new UnderflowException('Cannot dequeue a node from an empty queue.');
                    }

                    /**
                     * retrieve the highest priority.
                     *
                     * @var int
                     */
                    $priority = max_value(array_keys($this->queue));

                    /**
                     * retrieve the list of nodes with the priority `$priority`.
                     */
                    $nodes = $this->queue[$priority];

                    /**
                     * shift the first node out.
                     */
                    $node = array_shift($nodes);

                    /**
                     * If the list contained only this node, remove the list of nodes with priority `$priority`.
                     */
                    if ([] === $nodes) {
                        unset($this->queue[$priority]);

                        return $node;
                    }

                    $this->queue[$priority] = $nodes;

                    return $node;
                }

                /**
                 * Count the nodes in the queue.
                 *
                 * @return int<0, max>
                 *
                 * @psalm-mutation-free
                 */
                #[Override]
                public function count(): int
                {
                    $count = 0;
                    foreach ($this->queue as $list) {
                        $count += count($list);
                    }

                    /** @var int<0, max> */
                    return $count;
                }
            }
        "#},
    }
}
