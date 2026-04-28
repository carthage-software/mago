<?php

declare(strict_types=1);

/**
 * @template T
 */
final class GenInvBox2
{
    /** @param T $value */
    public function __construct(public mixed $value)
    {
    }

    /** @param T $v */
    public function set(mixed $v): void
    {
        $this->value = $v;
    }

    /** @return T */
    public function get(): mixed
    {
        return $this->value;
    }
}

/** @param GenInvBox2<int|string> $box */
function take_intstr_box(GenInvBox2 $box): void
{
}

/** @param GenInvBox2<int> $b */
function pass_int_to_intstr(GenInvBox2 $b): void
{
    /** @mago-expect analysis:invalid-argument */
    take_intstr_box($b);
}
