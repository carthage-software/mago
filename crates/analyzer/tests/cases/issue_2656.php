<?php

declare(strict_types=1);

namespace Issue2656;

/** @phpstan-type MaybeLeaf = Leaf|null */
final class Leaf
{
    public function __construct(public readonly ?string $value) {}
}

final class Middle
{
    public function __construct(private readonly ?Leaf $leaf) {}

    /** @pure */
    public function leaf(): ?Leaf
    {
        return $this->leaf;
    }

    public function other(): ?string
    {
        return null;
    }
}

final class Root
{
    public function __construct(private readonly ?Middle $middle) {}

    /** @pure */
    public function middle(): ?Middle
    {
        return $this->middle;
    }
}

final class UnstableRoot
{
    public function __construct(private ?Middle $middle) {}

    public function middle(): ?Middle
    {
        return $this->middle;
    }
}

/** @phpstan-assert !null $actual */
function ensureNotNull(mixed $actual): void {}

/** @phpstan-assert Leaf $actual */
function ensureLeaf(mixed $actual): void {}

/** @phpstan-assert Leaf|null $actual */
function ensureNullableLeaf(mixed $actual): void {}

/** @phpstan-import-type MaybeLeaf from Leaf */
final class Assertions
{
    /** @phpstan-assert MaybeLeaf $actual */
    public static function ensureMaybeLeaf(mixed $actual): void {}
}

/** @phpstan-assert !Leaf|null $actual */
function ensureNotLeafOrNull(mixed $actual): void {}

/**
 * @template T
 * @param T $expected
 * @phpstan-assert T $actual
 */
function ensureType(mixed $actual, mixed $expected): void {}

/** @phpstan-assert mixed $actual */
function ensureMixed(mixed $actual): void {}

/**
 * @pure
 * @phpstan-assert-if-true !null $actual
 * @phpstan-assert-if-true string $label
 */
function isNotNullWithLabel(mixed $actual, mixed $label): bool
{
    return $actual !== null && is_string($label);
}

/** @phpstan-assert-if-true !null $actual */
function isNotNull(mixed $actual): bool
{
    return $actual !== null;
}

/** @phpstan-assert-if-false !null $actual */
function isNull(mixed $actual): bool
{
    return $actual === null;
}

function problem(Root $root): ?string
{
    ensureNotNull($root->middle()?->leaf());

    return $root->middle()->other();
}

function problemProperty(Root $root): ?string
{
    ensureNotNull($root->middle()?->leaf()?->value);

    return $root->middle()->other();
}

function positiveTypeAssertion(Root $root): Middle
{
    ensureLeaf($root->middle()?->leaf());

    return $root->middle();
}

function positiveTemplateAssertion(Root $root, Leaf $expected): Middle
{
    ensureType($root->middle()?->leaf(), $expected);

    return $root->middle();
}

function negatedUnionAssertion(Root $root): Middle
{
    ensureNotLeafOrNull($root->middle()?->leaf()?->value);

    return $root->middle();
}

function multipleAssertions(Root $root, string $label): Middle
{
    if (isNotNullWithLabel($root->middle()?->leaf(), $label)) {
        return $root->middle();
    }

    return new Middle(null);
}

function conditionalTrueAssertion(Root $root): Middle
{
    if (isNotNull($root->middle()?->leaf())) {
        return $root->middle();
    }

    return new Middle(null);
}

function conditionalFalseAssertion(Root $root): Middle
{
    if (isNull($root->middle()?->leaf())) {
        return new Middle(null);
    }

    return $root->middle();
}

function conditionalAssertionDoesNotLeak(Root $root): Middle
{
    if (isNotNull($root->middle()?->leaf())) {
    }

    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $root->middle();
}

function impureReceiverStaysNullable(UnstableRoot $root): Middle
{
    ensureNotNull($root->middle()?->leaf());

    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $root->middle();
}

function nullableAssertionKeepsNullableReceiver(Root $root): Middle
{
    ensureNullableLeaf($root->middle()?->leaf());

    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $root->middle();
}

function nullableTemplateKeepsNullableReceiver(Root $root, ?Leaf $expected): Middle
{
    ensureType($root->middle()?->leaf(), $expected);

    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $root->middle();
}

function nullableAliasKeepsNullableReceiver(Root $root): Middle
{
    Assertions::ensureMaybeLeaf($root->middle()?->leaf());

    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $root->middle();
}

function mixedAssertionKeepsNullableReceiver(Root $root): Middle
{
    ensureMixed($root->middle()?->leaf());

    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $root->middle();
}

function unrelatedReceiverStaysNullable(Root $root, Root $other): Middle
{
    ensureNotNull($root->middle()?->leaf());

    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $other->middle();
}

function controlAssertCallDirectly(Root $root): ?string
{
    ensureNotNull($root->middle());

    return $root->middle()->other();
}

/** @throws \LogicException */
function controlIf(Root $root): ?string
{
    if ($root->middle()?->leaf() === null) {
        throw new \LogicException();
    }

    return $root->middle()->other();
}
