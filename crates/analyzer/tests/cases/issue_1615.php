<?php

declare(strict_types=1);

class Animal {}

class Dog extends Animal {}

abstract class AnimalShelter
{
    abstract public Animal $favorite { get; }
}

class DogShelter extends AnimalShelter
{
    public Dog $favorite {
        get => new Dog();
    }
}

interface HasName
{
    public Animal $mascot { get; }
}

class DogOwner implements HasName
{
    public Dog $mascot {
        get => new Dog();
    }
}

abstract class DogRegistry
{
    public Dog $latest {
        set {
            /* noop */
        }
    }
}

class AnimalRegistry extends DogRegistry
{
    public Animal $latest {
        set {
            /* noop */
        }
    }
}

abstract class StrictAnimalShelter
{
    abstract public Dog $favorite { get; }
}

class BadShelter extends StrictAnimalShelter
{
    /** @mago-expect analysis:incompatible-property-type */
    public Animal $favorite {
        get => new Animal();
    }
}

class RegularShelter
{
    public Animal $favorite;

    public function __construct()
    {
        $this->favorite = new Animal();
    }
}

class BadRegularShelter extends RegularShelter
{
    /** @mago-expect analysis:incompatible-property-type */
    public Dog $favorite;

    public function __construct()
    {
        $this->favorite = new Dog();
    }
}

abstract class BothHookShelter
{
    public Animal $favorite {
        get => new Animal();
        set {
            /* noop */
        }
    }
}

class BadBothHookShelter extends BothHookShelter
{
    /** @mago-expect analysis:incompatible-property-type */
    public Dog $favorite {
        get => new Dog();
        set {
            /* noop */
        }
    }
}
