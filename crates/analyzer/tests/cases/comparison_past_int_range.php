<?php

declare(strict_types=1);

function keeps_string(int|string $value): string
{
    if ($value > 9223372036854775807) {
        return $value;
    }

    return '';
}

function keeps_string_with_or_equal(int|string $value): string
{
    if ($value >= 9223372036854775807) {
        // @mago-expect analysis:invalid-return-statement
        return $value;
    }

    return '';
}
