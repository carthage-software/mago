<?php

declare(strict_types=1);

/**
 * @param callable(int): string $cb
 */
function run(callable $cb): string
{
    return $cb(1);
}

function two(int $a, int $b): string
{
    return (string) ($a + $b);
}

function one(int $a): string
{
    return (string) $a;
}

// @mago-expect analysis:possibly-invalid-argument
run('two');
run('one');
