<?php

declare(strict_types=1);

/**
 * @param callable(int): list<string> $cb
 */
function callables_run_to_list(callable $cb): int
{
    $result = $cb(3);
    return count($result);
}

// @mago-expect analysis:possibly-invalid-argument
echo callables_run_to_list(fn(int $n): array => array_fill(0, $n, 'x'));
