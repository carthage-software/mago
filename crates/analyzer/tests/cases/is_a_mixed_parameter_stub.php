<?php

declare(strict_types=1);

class TargetClass
{
    public function foo(): void {}
}

function test_is_a_mixed(mixed $val): bool
{
    return is_a($val, TargetClass::class);
}

function test_is_a_nullable(?TargetClass $val): bool
{
    return is_a($val, TargetClass::class);
}

function test_is_a_union(int|TargetClass $val): bool
{
    return is_a($val, TargetClass::class);
}

function test_is_a_narrowing(mixed $val): void
{
    if (is_a($val, TargetClass::class)) {
        $val->foo();
    }
}

function test_is_subclass_of_mixed(mixed $val): bool
{
    return is_subclass_of($val, TargetClass::class);
}

function test_is_subclass_of_nullable(?TargetClass $val): bool
{
    return is_subclass_of($val, TargetClass::class);
}

function test_is_subclass_of_union(int|TargetClass $val): bool
{
    return is_subclass_of($val, TargetClass::class);
}

function test_is_subclass_of_narrowing(mixed $val): void
{
    if (is_subclass_of($val, TargetClass::class, false)) {
        $val->foo();
    }
}
