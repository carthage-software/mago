<?php

declare(strict_types=1);

/** @return false|array<string, string> */
function fetch(): false|array
{
    return ['name' => 'test'];
}

function check(): bool
{
    return true;
}

// Variables assigned in the right operand of `&&` inside a while condition
// must be visible in the loop body with their narrowed types, not `mixed`.
// Both loops below should produce the same diagnostics.

function test_compound_while_condition(): void
{
    while (check() && ($row = fetch()) !== false) {
        /** @mago-expect analysis:possibly-undefined-string-array-index */
        $name = $row['name'];
    }
}

function test_simple_while_condition(): void
{
    while (($row = fetch()) !== false) {
        /** @mago-expect analysis:possibly-undefined-string-array-index */
        $name = $row['name'];
    }
}
