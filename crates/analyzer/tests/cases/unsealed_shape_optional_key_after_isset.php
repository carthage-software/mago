<?php

declare(strict_types=1);

/** @param array{name?: string, id: string, ...} $a */
function unsealed_shape_name_of(array $a): string
{
    if (isset($a['name'])) {
        echo 'has a name';
    }

    return $a['name'] ?? '';
}

/** @param array{name?: string, id: string, ...} $a */
function unsealed_shape_name_when_missing(array $a): string
{
    if (!isset($a['name'])) {
        return $a['id'];
    }

    return $a['name'];
}
