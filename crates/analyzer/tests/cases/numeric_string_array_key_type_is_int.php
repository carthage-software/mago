<?php

declare(strict_types=1);

/**
 * @param array<'123'|'234', float> $prop
 * @return array<'123'|'234', float>
 */
function copy_all(array $prop): array
{
    $out = [];
    foreach ($prop as $k => $v) {
        $out[$k] = $v;
    }

    return $out;
}

/** @return array<'123'|'234', float> */
function literal(): array
{
    return ['123' => 1.0];
}

/** @return array{'123': float} */
function shape(): array
{
    return ['123' => 1.0];
}
