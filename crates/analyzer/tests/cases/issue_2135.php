<?php

declare(strict_types=1);

class Animal {}

class Dog extends Animal {}

class Cat extends Animal {}

/**
 * @template TAnimal of Animal
 */
class Cage
{
    /** @var TAnimal|null */
    public ?Animal $occupant = null;
}

/**
 * @extends Cage<Dog>
 */
class DogCage extends Cage
{
    public function bark(): void {}
}

interface Handler
{
    /**
     * @template TAnimal of Animal
     *
     * @param Cage<TAnimal> $cage
     */
    public function handle(Cage $cage): void;
}

/**
 * @template TAnimal of Animal
 *
 * @param Cage<TAnimal> $cage
 */
function free_function(Cage $cage): ?DogCage
{
    if ($cage instanceof DogCage) {
        $cage->bark();

        return $cage;
    }

    return null;
}

class ImplementsHandler implements Handler
{
    public function handle(Cage $cage): void
    {
        if ($cage instanceof DogCage) {
            $cage->bark();
        }
    }
}

/**
 * @template TValue
 */
class Container
{
    /** @var TValue|null */
    public mixed $value = null;
}

/**
 * @extends Container<int>
 */
class IntContainer extends Container {}

/**
 * @template TValue
 *
 * @param Container<TValue> $container
 */
function unbounded(Container $container): ?IntContainer
{
    if ($container instanceof IntContainer) {
        return $container;
    }

    return null;
}

/**
 * @param Cage<Cat> $cage
 *
 * @mago-expect analysis:impossible-condition
 */
function incompatible_concrete(Cage $cage): void
{
    if ($cage instanceof DogCage) {
        $cage->bark();
    }
}
