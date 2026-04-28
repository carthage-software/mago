<?php

declare(strict_types=1);

/**
 * @template T
 */
final class GenSetter
{
    /** @var T */
    public mixed $value;

    /** @param T $v */
    public function __construct(mixed $v)
    {
        $this->value = $v;
    }

    /** @param T $v */
    public function set(mixed $v): void
    {
        $this->value = $v;
    }
}

/**
 * @param GenSetter<int> $s
 */
function set_int(GenSetter $s): void
{
    /** @mago-expect analysis:invalid-argument */
    $s->set('not int');
}
