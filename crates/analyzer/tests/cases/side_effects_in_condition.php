<?php

declare(strict_types=1);

function impure_function(): int
{
    return mt_rand(0, max: 100);
}

/** @pure */
function pure_function(int $x): int
{
    return $x * 2;
}

/** @psalm-mutation-free */
function mutation_free_function(int $x): int
{
    return $x + 1;
}

function test_impure_in_if(int $x): void
{
    /** @mago-expect analysis:side-effects-in-condition */
    if (impure_function() > 50) {
        echo 'big';
    }
}

function test_pure_in_if(int $x): void
{
    if (pure_function($x) > 50) {
        echo 'big';
    }
}

function test_mutation_free_in_if(int $x): void
{
    if (mutation_free_function($x) > 50) {
        echo 'big';
    }
}

function test_impure_in_while(): void
{
    /** @mago-expect analysis:side-effects-in-condition */
    while (impure_function() > 0) {
        break;
    }
}

function test_impure_in_ternary(): void
{
    /** @mago-expect analysis:side-effects-in-condition */
    $x = impure_function() > 50 ? 'yes' : 'no';
}

function test_impure_outside_condition(): void
{
    $val = impure_function();
    if ($val > 50) {
        echo 'big';
    }
}

function test_builtin_pure_in_condition(string $s): void
{
    if (strlen($s) > 10) {
        echo 'long';
    }
}

/** @param list<int> $arr */
function test_builtin_count_in_condition(array $arr): void
{
    if (count($arr) > 0) {
        echo 'non-empty';
    }
}
