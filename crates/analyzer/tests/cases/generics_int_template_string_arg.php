<?php

declare(strict_types=1);

/**
 * @template T of int
 */
final class GenIntCnstFE
{
    /** @var T */
    public mixed $value;

    /** @param T $v */
    public function __construct(mixed $v)
    {
        $this->value = $v;
    }
}

/** @mago-expect analysis:invalid-argument,template-constraint-violation */
new GenIntCnstFE('hello');
