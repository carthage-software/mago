<?php

declare(strict_types=1);

function takesClosure(\Closure $callback): void
{
    $callback();
}

function takesArray(array $_): void {}

function takesMixed(mixed $_): void {}

function arrayArm(array|callable $value): void
{
    if (\is_callable($value)) {
        $value();
        takesClosure($value(...));
        takesClosure(\Closure::fromCallable($value));
    }
}

function arrayOnly(array $value): void
{
    if (\is_callable($value)) {
        takesArray($value);
        takesMixed($value[0] ?? null);
        foreach ($value as $_) {
        }
        $value();
        takesClosure($value(...));
        takesClosure(\Closure::fromCallable($value));
    }
}

/** @param list<mixed> $value */
function listOnly(array $value): void
{
    if (\is_callable($value)) {
        takesArray($value);
        takesMixed($value[0] ?? null);
        foreach ($value as $_) {
        }
        $value();
        takesClosure($value(...));
        takesClosure(\Closure::fromCallable($value));
    }
}

function uncheckedArray(array $value): void
{
    // @mago-expect analysis:invalid-callable
    takesClosure($value(...));
    // @mago-expect analysis:less-specific-nested-argument-type
    takesClosure(\Closure::fromCallable($value));
}
