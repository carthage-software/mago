<?php

declare(strict_types=1);

/**
 * @template T
 */
final class GenWithCC
{
    public const NAME = 'gen';

    /** @param T $value */
    public function __construct(public mixed $value)
    {
    }
}

$g = new GenWithCC(1);
echo GenWithCC::NAME . ' ' . $g->value;
