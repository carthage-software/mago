<?php

declare(strict_types=1);

/**
 * @inheritors Cat|Dog
 */
abstract class Animal
{
    abstract public function makeSound(): string;
}

class Cat extends Animal
{
    public function meow(): string
    {
        return 'Meow!';
    }

    public function makeSound(): string
    {
        return $this->meow();
    }
}

class Dog extends Animal
{
    public function bark(): string
    {
        return 'Woof!';
    }

    public function makeSound(): string
    {
        return $this->bark();
    }
}

function getAnimalSound(Animal $animal): string
{
    if ($animal instanceof Cat) {
        return $animal->meow();
    }

    // After 1 check, we know it must be Dog
    return $animal->bark();
}
