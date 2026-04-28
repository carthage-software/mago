<?php

declare(strict_types=1);

function flow_assert_narrow(null|string $value): int
{
    assert($value !== null);

    return strlen($value);
}
