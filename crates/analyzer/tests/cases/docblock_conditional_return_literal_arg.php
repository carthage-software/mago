<?php

declare(strict_types=1);

/**
 * @template T of bool
 *
 * @param T $flag
 *
 * @return ($flag is true ? non-empty-string : null)
 */
function maybeStrBV(bool $flag): null|string
{
    return $flag ? 'yes' : null;
}

/** @param non-empty-string $s */
function takeNeBV(string $s): string
{
    return $s;
}

takeNeBV(maybeStrBV(true));
