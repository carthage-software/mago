<?php

declare(strict_types=1);

function loose_equality_zero_keeps_null(?int $value): string
{
    /** @mago-expect analysis:possibly-null-operand */
    if ($value == 0) {
        if ($value === null) {
            return 'null';
        }

        return 'zero';
    }

    return 'other';
}

function loose_equality_empty_string_keeps_null(?string $value): string
{
    /** @mago-expect analysis:possibly-null-operand */
    if ($value == '') {
        return $value ?? 'null';
    }

    return 'other';
}
