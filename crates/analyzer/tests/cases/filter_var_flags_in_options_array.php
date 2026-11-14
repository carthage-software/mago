<?php

declare(strict_types=1);

function global_ip(string $value): string
{
    $ip = filter_var($value, FILTER_VALIDATE_IP, ['flags' => FILTER_FLAG_GLOBAL_RANGE | FILTER_NULL_ON_FAILURE]);
    if ($ip === null) {
        return 'none';
    }

    return $ip;
}

/** @return array<array-key, mixed> */
function forced(string $value): array
{
    return filter_var($value, FILTER_VALIDATE_IP, FILTER_FORCE_ARRAY);
}

function required(mixed $value): string
{
    $ips = filter_var($value, FILTER_VALIDATE_IP, ['flags' => FILTER_REQUIRE_ARRAY]);
    if ($ips === false) {
        return 'none';
    }

    return 'array';
}
