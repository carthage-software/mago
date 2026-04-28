<?php

declare(strict_types=1);

/**
 * @template T
 */
final class GenWriter
{
    /** @var T */
    public mixed $value;

    /** @param T $v */
    public function __construct(mixed $v)
    {
        $this->value = $v;
    }

    /** @param T $v */
    public function write(mixed $v): void
    {
        $this->value = $v;
    }
}

/**
 * @param GenWriter<int> $w
 */
function call_writer(GenWriter $w): void
{
    $w->write(99);
}
