<?php

declare(strict_types=1);

final class Foo
{
    public int $n = 0;
}

function from_object(object $value, bool $flag): int
{
    if (!$value instanceof Foo) {
        return 0;
    } elseif ($flag) {
        return 1;
    }

    return $value->n;
}

/** @throws Exception */
function from_mixed(mixed $value, string $key): int
{
    if (!is_int($value)) {
        throw new Exception('Unexpected value');
    } elseif ($key !== 'foo') {
        throw new Exception('Unexpected key');
    }

    return $value * 3;
}

/** @return non-empty-string */
function from_string(string $value, bool $flag): string
{
    if ($value === '') {
        return 'x';
    } elseif ($flag) {
        echo 'flag';
    }

    return $value;
}
