<?php

declare(strict_types=1);

interface GenA
{
    public function a(): void;
}

interface GenB
{
    public function b(): void;
}

/**
 * @template T of GenA&GenB
 */
final class GenABHolder
{
    /** @param T $item */
    public function __construct(public GenA&GenB $item)
    {
    }

    /** @return T */
    public function get(): GenA&GenB
    {
        return $this->item;
    }
}

final class GenAB implements GenA, GenB
{
    public function a(): void
    {
    }

    public function b(): void
    {
    }
}

$h = new GenABHolder(new GenAB());
$h->get()->a();
$h->get()->b();
