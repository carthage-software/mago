<?php

declare(strict_types=1);

function take_int(int $_n): void {}

/**
 * @param array<string, string> $items
 */
function iterate(array $items): void
{
    foreach ($items as $v) {
        take_int($v);
    }
}
