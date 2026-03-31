<?php

declare(strict_types=1);

/**
 * @param list<array{bool,string}> $todo
 * @param-out list<array{bool,string}> $todo
 */
function test_bad(string $key, array &$todo): void
{
    /** @mago-expect analysis:reference-constraint-violation */
    $todo[] = $key;
}

/**
 * @param list<array{bool,string}> $todo
 * @param-out list<array{bool,string}> $todo
 */
function test_ok(string $key, array &$todo): void
{
    $todo[] = [true, $key];
}
