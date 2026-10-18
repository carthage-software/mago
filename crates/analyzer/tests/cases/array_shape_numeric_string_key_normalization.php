<?php

declare(strict_types=1);

/**
 * In PHP, array keys containing valid decimal integers are always cast to int,
 * so $data[10] and $data['10'] refer to the same element.
 *
 * @param array{10?: string} $data
 */
function array_key_exists_numeric_string_key(array $data): void
{
    if (array_key_exists('10', $data)) {
        accept_string($data[10]);
    }
}

/**
 * @param array{10?: string} $data
 */
function isset_numeric_string_key(array $data): void
{
    if (isset($data['10'])) {
        accept_string($data['10']);
    }
}

function accept_string(string $_): void {}
