<?php

declare(strict_types=1);

/** @return array{Min: mixed, Max: mixed}|false */
function getRow(): array|false
{
    return rand() ? false : ['Min' => 0, 'Max' => 1];
}

/** @param null|''|1 $_ */
function foo(int|string|null $_): void
{
}

$row = getRow();

if (
    $row === false
    || $row['Min'] === null
    || ($row['Min'] === '' && $row['Max'] === '')
    || $row['Min'] === 1
) {
    if (false !== $row) {
        foo($row['Min']);
    }
}

if (
    $row === false
    || $row['Min'] === null
    || $row['Min'] === 0
    || ($row['Min'] === '' && $row['Max'] === '')
) {
}
