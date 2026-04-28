<?php

declare(strict_types=1);

/**
 * @template T of int
 */
final class GenIntOnly
{
    /** @var T */
    public mixed $value;

    /** @param T $value */
    public function __construct(mixed $value)
    {
        $this->value = $value;
    }
}

/** @param GenIntOnly<int> $g */
function take_g_int(GenIntOnly $g): void
{
}

/** @mago-expect analysis:invalid-argument,template-constraint-violation */
take_g_int(new GenIntOnly('not an int'));
