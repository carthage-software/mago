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

/**
 * @template-covariant T
 */
interface CovariantNode
{
    /**
     * @return T
     */
    public function getValue(): mixed;
}

/**
 * @template-covariant T
 *
 * @implements CovariantNode<T>
 */
final readonly class CovariantTreeNode implements CovariantNode
{
    /**
     * @param T $value
     * @param list<CovariantNode<T>> $children
     */
    public function __construct(
        private mixed $value,
        private array $children = [],
    ) {}

    public function getValue(): mixed
    {
        return $this->value;
    }

    /**
     * @return list<CovariantNode<T>>
     */
    public function getChildren(): array
    {
        return $this->children;
    }
}

/**
 * @template T
 *
 * @param CovariantNode<T> $node
 */
function covariant_narrowing(CovariantNode $node): int
{
    $total = 1;
    if ($node instanceof CovariantTreeNode) {
        foreach ($node->getChildren() as $child) {
            $total += covariant_narrowing($child);
        }
    }

    return $total;
}
