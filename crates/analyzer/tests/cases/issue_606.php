<?php declare(strict_types=1);

/** @return array{a: int}|array{b: int}|array{a: int, b: int, c: bool} */
function get_one_of(): array
{
    return ['a' => 1];
}

$x = get_one_of();
if (isset($x['c'])) {
    echo "present\n";
}
