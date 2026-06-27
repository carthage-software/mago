<?php

declare(strict_types=1);

/** @template T */
class Box
{
    /** @param T $value */
    public function set(mixed $value): void {}

    /** @return T */
    public function get(): mixed
    {
        return null;
    }
}

class Animal {}

class Dog extends Animal {}

class Plant {}

/** @param Box<Animal> $box */
function invariant(Box $box): void {}

/** @param Box<covariant Animal> $box */
function covariantAnimal(Box $box): void {}

/** @param Box<covariant Dog> $box */
function covariantDog(Box $box): void {}

/** @param Box<contravariant Dog> $box */
function contravariantDog(Box $box): void {}

/**
 * @param Box<Dog> $dogBox
 * @param Box<Animal> $animalBox
 */
function accepted(Box $dogBox, Box $animalBox): void
{
    covariantAnimal($dogBox);
    covariantAnimal($animalBox);

    contravariantDog($dogBox);
    contravariantDog($animalBox);
}

/**
 * @param Box<Dog> $dogBox
 * @param Box<Animal> $animalBox
 * @param Box<Plant> $plantBox
 */
function rejected(Box $dogBox, Box $animalBox, Box $plantBox): void
{
    /** @mago-expect analysis:less-specific-argument */
    invariant($dogBox);

    /** @mago-expect analysis:less-specific-argument */
    covariantDog($animalBox);

    /** @mago-expect analysis:invalid-argument */
    contravariantDog($plantBox);
}
