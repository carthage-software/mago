<?php

declare(strict_types=1);

namespace Generics;

class Animal {}

class Dog extends Animal {}

class Cat extends Animal {}

class Puppy extends Dog {}

interface IFoo {}

interface IBar {}

/**
 * @template T
 */
interface Example
{
    /**
     * @param T $v
     * @return T
     */
    public function produce(mixed $v): mixed;
}

/**
 * @template-covariant T
 */
interface Producer
{
    /**
     * @return T
     */
    public function produce(): mixed;
}

/**
 * @template-contravariant T
 */
interface Sink
{
    /**
     * @param T $v
     */
    public function sink(mixed $v): void;
}

/**
 * @template T of object
 */
interface Tapper
{
    /**
     * @param T $object
     * @return T
     */
    public function tap(object $object): object;
}

/**
 * @template T of Animal
 */
interface AnimalBox
{
    /**
     * @return T
     */
    public function unwrap(): Animal;
}
