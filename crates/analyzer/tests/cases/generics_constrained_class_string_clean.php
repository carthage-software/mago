<?php

declare(strict_types=1);

interface GenAnimal2
{
}

final class GenCat implements GenAnimal2
{
}

/**
 * @template T of GenAnimal2
 */
final class GenZoo2
{
    /** @param class-string<T> $clz */
    public function __construct(public string $clz)
    {
    }
}

function ok_zoo(): GenZoo2
{
    return new GenZoo2(GenCat::class);
}
