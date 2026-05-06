<?php

declare(strict_types=1);

/**
 * @param-out non-empty-string $value
 */
function setNonEmptyBS(mixed &$value): void
{
    $value = 'hello';
}

$v = null;
setNonEmptyBS($v);

/** @param non-empty-string $s */
function takeNeBS(string $s): string
{
    return $s;
}

echo takeNeBS($v);
