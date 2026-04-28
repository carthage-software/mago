<?php

declare(strict_types=1);

/**
 * @template T
 */
final class GenBoxStr
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

/** @param GenBoxStr<string> $b */
function flow_str(GenBoxStr $b): void
{
    /** @mago-expect analysis:invalid-argument */
    $b->set(42);
}
