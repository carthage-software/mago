<?php

declare(strict_types=1);

function take_string(string $s): void
{
    echo $s;
}

/**
 * @param array{name?: string} $a
 */
function flow_isset_then_pass(array $a): void
{
    if (isset($a['name'])) {
        take_string($a['name']);
    }
}
