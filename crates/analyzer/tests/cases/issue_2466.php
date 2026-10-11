<?php

declare(strict_types=1);

namespace Issue2466;

function withDefault(callable|null $callback = null): callable
{
    $callback ??= function (): int {
        return 1;
    };

    return $callback;
}

function emptyBody(callable|null $callback = null): callable
{
    $callback ??= static function (): void {};

    return $callback;
}

function longForm(callable|null $callback = null): callable
{
    $callback = $callback ?? static function (): int {
        return 1;
    };

    return $callback;
}

function closureParameter(\Closure|null $callback = null): \Closure
{
    $callback ??= static function (): int {
        return 1;
    };

    return $callback;
}

function localVariable(bool $flag): callable
{
    $callback = $flag ? strlen(...) : null;
    $callback ??= static function (): int {
        return 1;
    };

    return $callback;
}

function intDefault(int|null $value = null): int
{
    $value ??= 1;

    return $value;
}

function withArrowFunction(callable|null $callback = null): callable
{
    $callback ??= static fn (): int => 1;

    return $callback;
}

function withFirstClassCallable(callable|null $callback = null): callable
{
    $callback ??= strlen(...);

    return $callback;
}

function withParentheses(callable|null $callback = null): callable
{
    $callback ??= (static function (): int {
        return 1;
    });

    return $callback;
}

function assignmentExpression(callable|null $callback = null): callable
{
    return $callback ??= static function (): int {
        return 1;
    };
}

function keepOriginalValue(string|null $value): \Closure|string
{
    $value ??= static function (): int {
        return 1;
    };

    if (is_string($value)) {
        return $value;
    }

    return $value;
}

function redundantWithDefault(callable $callback): callable
{
    // @mago-expect analysis:redundant-null-coalesce
    $callback ??= static function (): int {
        return 1;
    };

    return $callback;
}

function alwaysNull(): \Closure
{
    $callback = null;
    // @mago-expect analysis:redundant-null-coalesce
    $callback ??= static function (): int {
        return 1;
    };

    return $callback;
}
