<?php

declare(strict_types=1);

function nullable(?int $value): void
{
    if ($value) {
        if (!$value) { // @mago-expect analysis:impossible-condition
            echo 'never';
        }
    }
}

function int_or_false(int|false $value): void
{
    if ($value) {
        if (!$value) { // @mago-expect analysis:impossible-condition
            echo 'never';
        }
    }
}

/**
 * @param int<0, max> $value
 *
 * @return positive-int
 */
function positive_when_truthy(int $value): int
{
    if ($value) {
        return $value;
    }

    return 1;
}
