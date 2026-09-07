<?php

declare(strict_types=1);

final class FinalWithoutInvoke {}

class OpenWithoutInvoke {}

interface OpenInterfaceWithoutInvoke {}

final class FinalWithInvoke
{
    public function __invoke(): void {}
}

function acceptsCallable(callable $_): void {}

function rejectFinalClass(FinalWithoutInvoke $value): void
{
    // @mago-expect analysis:impossible-type-comparison
    if (is_callable($value)) {
    }
}

/** @param FinalWithoutInvoke|(callable(): void) $value */
function narrowFinalClass(mixed $value): void
{
    if (is_callable($value)) {
        acceptsCallable($value);
        $value();
    }
}

function narrowOpenClass(OpenWithoutInvoke $value): void
{
    if (is_callable($value)) {
        acceptsCallable($value);
        $value();
    }
}

function narrowOpenInterface(OpenInterfaceWithoutInvoke $value): void
{
    if (is_callable($value)) {
        acceptsCallable($value);
        $value();
    }
}

function narrowInvokableClass(FinalWithInvoke $value): void
{
    // @mago-expect analysis:redundant-type-comparison
    if (is_callable($value)) {
        acceptsCallable($value);
        $value();
    }
}

/** @mago-expect analysis:non-existent-class-like */
function narrowUnresolvedClass(\Totally\Gone\Klass $value): void
{
    if (is_callable($value)) {
        acceptsCallable($value);
        $value();
    }
}

/** @mago-expect analysis:non-existent-class-like */
function narrowNullableUnresolvedClass(?\Totally\Gone\NullableKlass $value): void
{
    if (is_callable($value)) {
        acceptsCallable($value);
        $value();
    }
}

// @mago-expect analysis:non-existent-class-like
/**
 * @param \Totally\Gone\UnionKlass|(callable(): void) $value
 */
function narrowUnresolvedClassUnion(mixed $value): void
{
    if (is_callable($value)) {
        acceptsCallable($value);
        $value();
    }
}

final class UnresolvedPropertyHolder
{
    // @mago-expect analysis:non-existent-class-like
    public ?\Totally\Gone\PropertyKlass $value = null;
}

function narrowNullableUnresolvedProperty(UnresolvedPropertyHolder $holder): void
{
    if (is_callable($holder->value)) {
        acceptsCallable($holder->value);
        ($holder->value)();
    }
}
