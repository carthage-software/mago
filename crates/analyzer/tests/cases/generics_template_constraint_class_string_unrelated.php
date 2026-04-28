<?php

declare(strict_types=1);

interface GenAnimalCs2
{
}

final class GenDogCs2 implements GenAnimalCs2
{
}

final class GenCarCs2
{
}

/**
 * @template T of GenAnimalCs2
 *
 * @param class-string<T> $clz
 *
 * @return class-string<T>
 */
function gen_id_animal_cs(string $clz): string
{
    return $clz;
}

/** @mago-expect analysis:invalid-argument */
gen_id_animal_cs(GenCarCs2::class);
