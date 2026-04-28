<?php

declare(strict_types=1);

interface GenAnimal
{
}

final class GenDog implements GenAnimal
{
}

final class GenPlant
{
}

/**
 * @template T of GenAnimal
 */
final class GenZoo
{
    /** @param class-string<T> $clz */
    public function __construct(public string $clz)
    {
    }
}

/** @mago-expect analysis:invalid-argument */
function bad_zoo(): GenZoo
{
    return new GenZoo(GenPlant::class);
}
