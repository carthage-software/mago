<?php

class Animal
{
    public function name(): string
    {
        return 'animal';
    }
}

class Dog extends Animal
{
    public function fetch(): string
    {
        return 'stick';
    }
}

class Shelter
{
    public function adopt(): Animal
    {
        return new Animal();
    }
}

class DogShelter extends Shelter
{
}
