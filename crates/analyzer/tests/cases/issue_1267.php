<?php declare(strict_types=1);

/**
 * @return array{inner: array{a: int}|array{b: int}}
 */
function get_information(string $hash): array
{
    return get_information($hash);
}

$info = get_information('some-hash');
$z = $info['inner']['b'] ?? 0;

function take_int(int $cost): void
{
    echo $cost;
}

take_int($z);
