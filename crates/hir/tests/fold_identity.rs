use std::borrow::Cow;

use mago_allocator::LocalArena;

use mago_database::file::File;
use mago_hir::fold::Fold;
use mago_hir::ir::IR;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::item::annotation::ItemAnnotation;
use mago_hir::ir::item::annotation::generics::Variance;
use mago_hir::ir::item::annotation::member::PropertyAnnotationKind;
use mago_hir::ir::item::expression::ItemExpressionKind;
use mago_hir::ir::item::member::MemberItemKind;
use mago_hir::ir::item::statement::ItemStatementKind;
use mago_hir::ir::statement::StatementKind;
use mago_hir::lower::LowerSettings;
use mago_hir::lower::Lowering;
use mago_hir::walker::MutWalker;
use mago_span::Span;
use mago_syntax::parser::parse_file;

struct Identity<'arena> {
    arena: &'arena LocalArena,
}

impl<'arena> Fold<'_, 'arena, LocalArena> for Identity<'arena> {
    type FromItem = ();
    type FromStatement = ();
    type FromExpression = ();
    type ToItem = ();
    type ToStatement = ();
    type ToExpression = ();

    fn arena(&self) -> &'arena LocalArena {
        self.arena
    }

    fn fold_statement_meta(
        &self,
        _span: Span,
        _kind: &StatementKind<'arena, Self::ToItem, Self::ToStatement, Self::ToExpression>,
    ) -> Self::ToStatement {
    }

    fn fold_expression_meta(
        &self,
        _span: Span,
        _kind: &ExpressionKind<'arena, Self::ToItem, Self::ToStatement, Self::ToExpression>,
    ) -> Self::ToExpression {
    }

    fn fold_item_statement_meta(
        &self,
        _span: Span,
        _kind: &ItemStatementKind<'arena, Self::ToItem, Self::ToStatement, Self::ToExpression>,
    ) -> Self::ToItem {
    }

    fn fold_item_expression_meta(
        &self,
        _span: Span,
        _kind: &ItemExpressionKind<'arena, Self::ToItem, Self::ToStatement, Self::ToExpression>,
    ) -> Self::ToItem {
    }

    fn fold_member_item_meta(
        &self,
        _span: Span,
        _kind: &MemberItemKind<'arena, Self::ToItem, Self::ToStatement, Self::ToExpression>,
    ) -> Self::ToItem {
    }
}

const FIXTURE: &str = r#"<?php

declare(strict_types=1);

namespace App;

use RuntimeException;

const LIMIT = 100;

/**
 * A base collection.
 *
 * @template T of object
 * @template-covariant U = int
 * @type Pair = array{first: T, second: U}
 * @import-type Other from \App\Source as Imported
 * @extends Base<T>
 * @implements \IteratorAggregate<int, T>
 * @require-extends Base
 * @use Helper<T>
 * @sealed Collection|EmptyCollection
 * @mixin \App\Macroable
 * @method static T make(int $size = 1, string ...$names)
 * @property-read int $count
 * @property-write string $label
 */
final class Collection extends Base implements \IteratorAggregate
{
    use Helper;

    /** @var non-empty-string */
    public const string NAME = 'collection';

    /** @var list<T> */
    private array $items = [];

    public int $total = 0 {
        get => $this->total;
        set(int $value) {
            $this->total = $value;
        }
    }

    /**
     * @param T $input
     * @param-out U $output
     * @param (callable(T): bool)|null $predicate
     * @return list<T> [first, second]
     * @throws RuntimeException
     * @assert truthy $input
     * @assert-if-true non-empty $this->items
     * @assert-if-false !null $input->value()
     * @where U is int
     * @self-out static
     */
    public function process(mixed $input, mixed &$output, ?callable $predicate = null): array
    {
        global $registry;
        static $cache = null;

        try {
            $closure = function () use ($input): int {
                return \strlen((string) $input);
            };
            $arrow = static fn(int $index): int => $index + 1;
            $callable = $closure(...);
            $object = new class extends Base {
                public function id(): string
                {
                    return 'anonymous';
                }
            };

            $output = match (true) {
                $closure() > LIMIT => yield from [1, 2, 3],
                default => $arrow(1) <=> 2,
            };
        } catch (RuntimeException | \LogicException $error) {
            throw $error;
        } finally {
            unset($cache);
        }

        for ($index = 0; $index < 3; $index++) {
            continue;
        }

        foreach ($this->items as $key => $item) {
            if ($key > 1) {
                break;
            }
        }

        while (false) {
            do {
                echo "interpolated {$input} string";
            } while (false);
        }

        switch ($this->total) {
            case 1:
                return [];
            default:
                return [$input?->value ?? null, $this->items[0] ?? null, ...$this->items];
        }
    }
}

interface Base
{
    public function id(): string;
}

trait Helper
{
    public function help(): void
    {
    }
}

enum Suit: string implements \JsonSerializable
{
    case Hearts = 'h';
    case Spades = 's';

    public function jsonSerialize(): mixed
    {
        return $this->value;
    }
}

function entry(Suit $suit = Suit::Hearts): \Generator
{
    yield $suit->value => LIMIT;
}
"#;

#[derive(Default)]
struct Coverage {
    annotations: usize,
    type_parameters: usize,
    covariant_type_parameters: usize,
    methods: usize,
    method_parameter_defaults: usize,
    read_properties: usize,
    write_properties: usize,
    asserts: usize,
    asserts_if_true: usize,
    asserts_if_false: usize,
    sealings: usize,
    annotation_errors: usize,
}

impl<'arena> MutWalker<'arena, (), (), (), ()> for Coverage {
    fn walk_in_item_annotation(&mut self, annotation: &ItemAnnotation<'arena, (), (), ()>, _context: &mut ()) {
        self.annotations += 1;
        self.type_parameters += annotation.type_parameters.len();
        self.covariant_type_parameters += annotation
            .type_parameters
            .iter()
            .filter(|type_parameter| type_parameter.variance == Variance::Covariant)
            .count();
        self.methods += annotation.methods.len();
        self.method_parameter_defaults += annotation
            .methods
            .iter()
            .flat_map(|method| method.parameters.iter())
            .filter(|parameter| parameter.default_value.is_some())
            .count();
        self.read_properties +=
            annotation.properties.iter().filter(|property| property.kind == PropertyAnnotationKind::Read).count();
        self.write_properties +=
            annotation.properties.iter().filter(|property| property.kind == PropertyAnnotationKind::Write).count();
        self.asserts += annotation.asserts.len();
        self.asserts_if_true += annotation.asserts_if_true.len();
        self.asserts_if_false += annotation.asserts_if_false.len();
        self.sealings += annotation.sealings.len();
        self.annotation_errors += annotation.errors.len();
    }
}

#[test]
fn identity_fold_preserves_the_entire_lowered_ir() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"fixture.php"), Cow::Owned(FIXTURE.as_bytes().to_vec()));
    let program = parse_file(&scratch, &file);
    let ir: IR<'_, (), (), ()> = Lowering::new(&arena, &scratch, &file, program, LowerSettings::default()).lower();
    drop(scratch);

    assert!(!ir.statements.is_empty(), "the fixture must lower to a non-empty IR");
    assert!(ir.errors.is_empty(), "the fixture must lower without errors, got {:?}", ir.errors);

    let mut coverage = Coverage::default();
    coverage.walk_ir(&ir, &mut ());

    assert_eq!(coverage.annotation_errors, 0, "the fixture docblocks must lower without annotation errors");
    assert!(coverage.annotations >= 4, "expected class, property, constant, and method annotations");
    assert_eq!(coverage.type_parameters, 2, "expected the class templates to lower");
    assert_eq!(coverage.covariant_type_parameters, 1, "expected `@template-covariant` to carry its variance");
    assert_eq!(coverage.methods, 1, "expected the `@method` annotation to lower");
    assert_eq!(coverage.method_parameter_defaults, 1, "expected the `@method` parameter default to lower");
    assert_eq!(coverage.read_properties, 1, "expected `@property-read` to lower");
    assert_eq!(coverage.write_properties, 1, "expected `@property-write` to lower");
    assert_eq!(coverage.asserts, 1, "expected `@assert` to lower");
    assert_eq!(coverage.asserts_if_true, 1, "expected `@assert-if-true` to lower");
    assert_eq!(coverage.asserts_if_false, 1, "expected `@assert-if-false` to lower");
    assert_eq!(coverage.sealings, 1, "expected `@sealed` to lower");

    let identity = Identity { arena: &arena };
    let folded = identity.fold_ir(&ir);

    assert_eq!(folded, ir, "an identity fold must preserve the IR structurally");
}

#[test]
fn fold_deep_copies_the_ir_onto_a_different_arena() {
    let source_arena = LocalArena::new();
    let target_arena = LocalArena::new();
    let scratch = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"fixture.php"), Cow::Owned(FIXTURE.as_bytes().to_vec()));
    let program = parse_file(&scratch, &file);
    let ir: IR<'_, (), (), ()> =
        Lowering::new(&source_arena, &scratch, &file, program, LowerSettings::default()).lower();
    drop(scratch);

    let identity = Identity { arena: &target_arena };
    let folded = identity.fold_ir(&ir);

    assert_eq!(folded, ir, "a fold onto a fresh arena must preserve the IR structurally");

    drop(source_arena);

    let mut coverage = Coverage::default();
    coverage.walk_ir(&folded, &mut ());

    assert!(coverage.annotations >= 4, "the folded IR must keep the annotations after its source arena is dropped");
    assert_eq!(coverage.type_parameters, 2, "the folded IR must keep the class templates");
    assert_eq!(coverage.methods, 1, "the folded IR must keep the `@method` annotation");
    assert_eq!(coverage.read_properties, 1, "the folded IR must keep `@property-read`");
    assert_eq!(coverage.asserts, 1, "the folded IR must keep `@assert`");
    assert_eq!(coverage.sealings, 1, "the folded IR must keep `@sealed`");
}

#[test]
fn document_errors_are_lowered_onto_the_annotation_and_survive_the_fold() {
    const ERRORED: &str = "<?php

/**
 * Some `unclosed inline code.
 *
 * @param int< $size
 */
function broken(int $size): void
{
}
";

    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"errored.php"), Cow::Owned(ERRORED.as_bytes().to_vec()));
    let program = parse_file(&scratch, &file);
    let ir: IR<'_, (), (), ()> = Lowering::new(&arena, &scratch, &file, program, LowerSettings::default()).lower();
    drop(scratch);

    let mut coverage = Coverage::default();
    coverage.walk_ir(&ir, &mut ());

    assert!(
        coverage.annotation_errors >= 2,
        "expected the docblock parse errors to be lowered onto the annotation, got {}",
        coverage.annotation_errors
    );

    let identity = Identity { arena: &arena };
    let folded = identity.fold_ir(&ir);

    assert_eq!(folded, ir, "an identity fold must preserve annotation errors");
}
