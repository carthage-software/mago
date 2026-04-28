<?php

declare(strict_types=1);

/**
 * @template T
 */
final class GenVoidT
{
    /** @param T $value */
    public function __construct(public mixed $value)
    {
    }

    /** @param T $v */
    public function add(mixed $v): void
    {
        $this->value = $v;
    }
}

/** @param GenVoidT<int> $v */
function set_int_void(GenVoidT $v): void
{
    /** @mago-expect analysis:invalid-argument */
    $v->add('not int');
}
