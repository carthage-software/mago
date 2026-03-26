<?php

declare(strict_types=1);

/**
 * @param list<int> $stack
 */
function test_end(array $stack): void
{
    while ($stack !== []) {
        $node = end($stack);

        echo $node + 1;
    }
}

/**
 * @param list<int> $stack
 */
function test_array_pop(array $stack): void
{
    while ($stack !== []) {
        $node = array_pop($stack);

        echo $node + 1;
    }
}

/**
 * @param list<int> $stack
 */
function test_mixed_operations(array $stack): void
{
    while ($stack !== []) {
        $node = end($stack);

        array_pop($stack);

        echo $node + 1;
    }
}
