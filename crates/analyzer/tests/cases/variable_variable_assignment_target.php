<?php

declare(strict_types=1);

/** @param array<string, int> $values */
function import(array $values): void
{
    foreach ($values as $key => $value) {
        $$key = $value;
        ${$key} = $value;
    }
}

function nested(string $name): void
{
    $other = $name;
    $$$other = 1;
}
