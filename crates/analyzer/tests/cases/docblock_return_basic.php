<?php

declare(strict_types=1);

/**
 * @return non-empty-string
 */
function greeting(): string
{
    return 'hello';
}

/**
 * @return int<1, 100>
 */
function dice_roll(): int
{
    return 1;
}

echo greeting();
echo dice_roll();

/** @param non-empty-string $s */
function takeNeR(string $s): string
{
    return $s;
}

/**
 * @return string
 */
function maybeEmpty(): string
{
    return '';
}

/** @mago-expect analysis:possibly-invalid-argument */
takeNeR(maybeEmpty());
