<?php

declare(strict_types=1);

class GenAnimalCo
{
}

final class GenDogCo extends GenAnimalCo
{
}

/**
 * @template-contravariant T
 */
interface GenComparator
{
    /**
     * @param T $a
     * @param T $b
     */
    public function compare(mixed $a, mixed $b): int;
}

/**
 * @implements GenComparator<GenAnimalCo>
 */
final class GenAnimalComparator implements GenComparator
{
    public function compare(mixed $a, mixed $b): int
    {
        return 0;
    }
}

/**
 * @param GenComparator<GenDogCo> $cmp
 */
function uses_dog_comparator(GenComparator $cmp): int
{
    return $cmp->compare(new GenDogCo(), new GenDogCo());
}

uses_dog_comparator(new GenAnimalComparator());
