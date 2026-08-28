<?php

declare(strict_types=1);

namespace Fixtures\Callables;

function shout(string $value): string
{
    return $value . '!';
}

final class Formatter
{
    public static function format(string $value): string
    {
        return $value;
    }
}

function accept(?callable $callback): void
{
    if (null !== $callback) {
        $callback('x');
    }
}

/** @param callable(string): string $callback */
function apply(callable $callback, string $input): string
{
    return $callback($input);
}

function takes_int(int $_i): void {}

// A fully qualified callable string resolves like its unqualified form.
accept('trim');
accept('\trim');
accept('Fixtures\Callables\shout');
accept('\Fixtures\Callables\shout');
accept('Fixtures\Callables\Formatter::format');
accept('\Fixtures\Callables\Formatter::format');
accept(['Fixtures\Callables\Formatter', 'format']);
accept(['\Fixtures\Callables\Formatter', 'format']);

echo apply('\strtoupper', 'a');
echo apply('\Fixtures\Callables\shout', 'a');
echo apply('\Fixtures\Callables\Formatter::format', 'a');
echo apply(['\Fixtures\Callables\Formatter', 'format'], 'a');

// The alias is truly resolved, so the return type is known.
function test_return_type_of_qualified_callable_string(): void
{
    $fn = '\strtoupper';

    /** @mago-expect analysis:invalid-argument */
    takes_int($fn('a'));
}

function test_invalid_callable_string(): void
{
    $fn = '\strouper';

    /** @mago-expect analysis:mixed-argument,non-existent-function */
    takes_int($fn('a'));
}

function test_empty_callable_string(): void
{
    $fn = '';

    /** @mago-expect analysis:mixed-argument,non-existent-function */
    takes_int($fn('a'));
}

function test_empty_qualified_callable_string(): void
{
    $fn = '\\';

    /** @mago-expect analysis:mixed-argument,non-existent-function */
    takes_int($fn('a'));
}
