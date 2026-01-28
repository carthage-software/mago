<?php

declare(strict_types=1);

/** @param array{id: int, ...} $x */
function takes_array_with_int_id(array $x): void
{
    takes_array_with_int_id($x);
}

/** @var mixed $x */
if (!isset($x['id']) || !is_int($x['id'])) {
    die("missing or invalid 'id'");
}

takes_array_with_int_id($x);
