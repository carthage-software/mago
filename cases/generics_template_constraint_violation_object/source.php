<?php

declare(strict_types=1);

interface GenSized2
{
    public function size(): int;
}

/**
 * @template T of GenSized2
 */
final class GenSizedHolder2
{
    /** @param T $item */
    public function __construct(
        public GenSized2 $item,
    ) {}
}

final class GenNotSized
{
    public int $x = 0;
}

new GenSizedHolder2(new GenNotSized());
