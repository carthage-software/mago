<?php

declare(strict_types=1);

interface GenSized
{
    public function size(): int;
}

/**
 * @template T of GenSized
 */
final class GenSizedHolder
{
    /** @param T $item */
    public function __construct(public GenSized $item)
    {
    }

    /** @return T */
    public function get(): GenSized
    {
        return $this->item;
    }
}

final class GenSizedBag implements GenSized
{
    public function size(): int
    {
        return 5;
    }
}

function takes_sized(GenSized $s): int
{
    return $s->size();
}

takes_sized((new GenSizedHolder(new GenSizedBag()))->get());
