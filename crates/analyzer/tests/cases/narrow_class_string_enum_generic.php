<?php

declare(strict_types=1);

namespace Fruits;

/**
 * @template-covariant T
 * @template-covariant E
 * @inheritors Ripe|Spoiled
 * @phpstan-sealed Ripe|Spoiled
 */
abstract class Basket
{
    /** @return T */
    abstract public function fruit();

    /** @return E */
    abstract public function spoilage();
}

/**
 * @template-covariant T
 * @template-covariant E
 * @extends Basket<T, E>
 */
final class Ripe extends Basket
{
    /** @param T $fruit */
    public function __construct(
        private mixed $fruit,
    ) {}

    /** @return T */
    public function fruit()
    {
        return $this->fruit;
    }

    /** @return never */
    public function spoilage(): never
    {
        exit();
    }
}

/**
 * @template-covariant T
 * @template-covariant E
 * @extends Basket<T, E>
 */
final class Spoiled extends Basket
{
    /** @param E $spoilage */
    public function __construct(
        private mixed $spoilage,
    ) {}

    /** @return never */
    public function fruit(): never
    {
        exit();
    }

    /** @return E */
    public function spoilage()
    {
        return $this->spoilage;
    }
}

final class Apple {}

enum Spoilage
{
    case Bruised;
    case Moldy;
}

final class Worm {}

function eat(Apple $apple): void {}

/** Enum as the second type argument: BROKEN — fruit() is inferred as `mixed`. */
/** @param Basket<Apple, Spoilage> $basket */
function with_enum_argument(Basket $basket): void
{
    if ($basket instanceof Ripe) {
        eat($basket->fruit());
    }
}

/** Class as the second type argument: works — fruit() is correctly `Apple`. */
/** @param Basket<Apple, Worm> $basket */
function with_class_argument(Basket $basket): void
{
    if ($basket instanceof Ripe) {
        eat($basket->fruit());
    }
}
