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

function needsAnimal(Animal $animal): void {}

function needsString(string $value): void {}

function needsMixed(mixed $value): void {}

/**
 * @param Box<covariant Animal> $covariant
 * @param Box<contravariant Dog> $contravariant
 * @param Box<*> $star
 */
function projections(Box $covariant, Box $contravariant, Box $star): void
{
    needsAnimal($covariant->get());

    /** @mago-expect analysis:possibly-invalid-argument */
    $covariant->set(new Dog());

    $contravariant->set(new Dog());

    /** @mago-expect analysis:mixed-argument */
    needsString($contravariant->get());

    needsMixed($star->get());

    /** @mago-expect analysis:possibly-invalid-argument */
    $star->set(new Dog());
}

/** @param Box<*> $star */
function acceptsAnything(Box $star): void {}

/**
 * @param Box<Dog> $dogBox
 * @param Box<Animal> $animalBox
 */
function starAssignability(Box $dogBox, Box $animalBox): void
{
    acceptsAnything($dogBox);
    acceptsAnything($animalBox);
}
