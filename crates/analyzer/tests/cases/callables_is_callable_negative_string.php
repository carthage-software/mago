<?php

declare(strict_types=1);

namespace Fixtures\Callables;

function custom_fn(): void
{
}

/** @param array<int> $a */
function accept_array(array $a): void
{
    array_pop($a);
}

/** @param 'not_a_function' $s */
function accept_missing_function(string $s): void
{
    echo $s;
}

/** @param 'strlen'|'\strlen'|'StrLen'|'\StrLen'|list<int> $value */
function test_global_function(mixed $value): void
{
    if (!is_callable($value)) {
        accept_array($value);
    }
}

/** @param 'Fixtures\Callables\custom_fn'|'\Fixtures\Callables\custom_fn'|'Fixtures\Callables\Custom_Fn'|'\Fixtures\Callables\Custom_Fn'|list<int> $value */
function test_namespaced_function(mixed $value): void
{
    if (!is_callable($value)) {
        accept_array($value);
    }
}

/** @param 'strlen'|'not_a_function' $value */
function test_missing_function_narrowing(string $value): void
{
    if (!is_callable($value)) {
        accept_missing_function($value);
    }
}

/** @param '\strlen'|'not_a_function'|list<int> $value */
function test_missing_function_remains_in_union(mixed $value): void
{
    if (!is_callable($value)) {
        // @mago-expect analysis:possibly-invalid-argument
        accept_array($value);
    }
}

test_global_function('strlen');
test_global_function('\strlen');
test_global_function('StrLen');
test_global_function('\StrLen');
test_global_function([1]);

test_namespaced_function('Fixtures\Callables\custom_fn');
test_namespaced_function('\Fixtures\Callables\custom_fn');
test_namespaced_function('Fixtures\Callables\Custom_Fn');
test_namespaced_function('\Fixtures\Callables\Custom_Fn');
test_namespaced_function([1]);

test_missing_function_narrowing('strlen');
test_missing_function_narrowing('not_a_function');

test_missing_function_remains_in_union('\strlen');
test_missing_function_remains_in_union('not_a_function');
test_missing_function_remains_in_union([1]);
