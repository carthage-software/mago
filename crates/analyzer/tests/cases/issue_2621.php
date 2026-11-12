<?php

declare(strict_types=1);

namespace Issue2621;

final class Foo
{
    public function __construct(public ?int $value) {}

    public function bar(): ?int
    {
        return $this->value;
    }

    public function withArgument(mixed $_): ?int
    {
        return $this->value;
    }

    /** @param-out null $receiver */
    public function withReference(?Foo &$receiver): ?int
    {
        $receiver = null;

        return $this->value;
    }
}

function takesFoo(Foo $_): void {}

function takesInt(int $_): void {}

function methodNotNull(?Foo $foo): void
{
    // @mago-expect analysis:possible-method-access-on-null
    if ($foo->bar() !== null) {
        takesFoo($foo);
    }
}

function propertyNotNull(?Foo $foo): void
{
    // @mago-expect analysis:possibly-null-property-access
    if ($foo->value !== null) {
        takesFoo($foo);
    }
}

function methodEarlyReturn(?Foo $foo): void
{
    // @mago-expect analysis:possible-method-access-on-null
    if ($foo->bar() === null) {
        return;
    }

    takesFoo($foo);
}

function propertyEarlyReturn(?Foo $foo): void
{
    // @mago-expect analysis:possibly-null-property-access
    if (null === $foo->value) {
        return;
    }

    takesFoo($foo);
}

function truthyMethod(?Foo $foo): void
{
    // @mago-expect analysis:possible-method-access-on-null
    if ($foo->bar()) {
        takesFoo($foo);
    }
}

function truthyProperty(?Foo $foo): void
{
    // @mago-expect analysis:possibly-null-property-access
    if ($foo->value) {
        takesFoo($foo);
    }
}

function methodEqualsValue(?Foo $foo): void
{
    // @mago-expect analysis:possible-method-access-on-null
    if ($foo->bar() === 1) {
        takesFoo($foo);
    }
}

function propertyEqualsValue(?Foo $foo): void
{
    // @mago-expect analysis:possibly-null-property-access
    if (1 === $foo->value) {
        takesFoo($foo);
    }
}

function storedMethodResult(?Foo $foo): void
{
    // @mago-expect analysis:possible-method-access-on-null
    $value = $foo->bar();
    if ($value !== null) {
        takesFoo($foo);
    }
}

function storedPropertyValue(?Foo $foo): void
{
    // @mago-expect analysis:possibly-null-property-access
    $value = $foo->value;
    if ($value !== null) {
        takesFoo($foo);
    }
}

function ternaryProperty(?Foo $foo): void
{
    // @mago-expect analysis:possibly-null-property-access
    $foo->value !== null ? takesFoo($foo) : null;
}

function propertyComparisonWithUnrelatedFlag(?Foo $foo, bool $flag): void
{
    // @mago-expect analysis:possibly-null-property-access
    if ($foo->value !== null || $flag) {
        // @mago-expect analysis:possibly-null-argument
        takesFoo($foo);
    }
}

function nullPropertyKeepsNullableReceiver(?Foo $foo): void
{
    // @mago-expect analysis:possibly-null-property-access
    if ($foo->value === null) {
        // @mago-expect analysis:possibly-null-argument
        takesFoo($foo);
    }
}

function nullablePropertyFalseBranch(?Foo $foo): void
{
    // @mago-expect analysis:possibly-null-property-access
    if ($foo->value !== null) {
        takesFoo($foo);
    } else {
        // @mago-expect analysis:possibly-null-argument
        takesFoo($foo);
    }
}

function unrelatedReceiver(?Foo $foo, ?Foo $other): void
{
    // @mago-expect analysis:possible-method-access-on-null
    if ($foo->bar() !== null) {
        // @mago-expect analysis:possibly-null-argument
        takesFoo($other);
    }
}

function repeatedImpureGetter(?Foo $foo): void
{
    // @mago-expect analysis:possible-method-access-on-null
    if ($foo->bar() !== null) {
        // @mago-expect analysis:possibly-null-argument
        takesInt($foo->bar());
    }
}

function nullsafeControl(?Foo $foo): void
{
    if ($foo?->bar() !== null) {
        takesFoo($foo);
    }
}

function unequalPropertyMayHaveNullReceiver(?Foo $foo): void
{
    // @mago-expect analysis:possibly-null-property-access
    if ($foo->value !== 1) {
        // @mago-expect analysis:possibly-null-argument
        takesFoo($foo);
    }
}

function equalityAfterEarlyReturn(?Foo $foo): void
{
    // @mago-expect analysis:possibly-null-property-access
    if ($foo->value !== 1) {
        return;
    }

    takesFoo($foo);
}

function negatedIsNull(?Foo $foo): void
{
    // @mago-expect analysis:possible-method-access-on-null
    if (!is_null($foo->bar())) {
        takesFoo($foo);
    }
}

function propertyMatch(?Foo $foo): void
{
    // @mago-expect analysis:possibly-null-property-access
    match ($foo->value) {
        1 => takesFoo($foo),
        // @mago-expect analysis:possibly-null-argument
        null => takesFoo($foo),
        default => null,
    };
}

function reassignedReceiver(?Foo $foo, ?Foo $other): void
{
    // @mago-expect analysis:possibly-null-property-access
    $value = $foo->value;
    $foo = $other;
    if ($value !== null) {
        // @mago-expect analysis:possibly-null-argument
        takesFoo($foo);
    }
}

function receiverReassignedByArgument(?Foo $foo): void
{
    // @mago-expect analysis:possible-method-access-on-null
    if ($foo->withArgument($foo = null) !== null) {
        // @mago-expect analysis:null-argument
        takesFoo($foo);
    }
}

function literalArgument(?Foo $foo): void
{
    // @mago-expect analysis:possible-method-access-on-null
    if ($foo->withArgument(1) !== null) {
        takesFoo($foo);
    }
}

function receiverChangedByReference(?Foo $foo): void
{
    // @mago-expect analysis:possible-method-access-on-null
    if ($foo->withReference($foo) !== null) {
        // @mago-expect analysis:null-argument
        takesFoo($foo);
    }
}

/**
 * @param-out null $receiver
 * @return 'value'
 */
function propertyName(?Foo &$receiver): string
{
    $receiver = null;

    return 'value';
}

function receiverChangedByPropertyName(?Foo $foo): void
{
    // @mago-expect analysis:possibly-null-property-access
    if ($foo->{propertyName($foo)} !== null) {
        // @mago-expect analysis:null-argument
        takesFoo($foo);
    }
}
