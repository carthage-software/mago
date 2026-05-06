<?php

declare(strict_types=1);

/**
 * @psalm-inheritors Apple|Orange|Banana|Grape|Mango
 */
abstract class Fruit
{
    abstract public function getColor(): string;
}

class Apple extends Fruit
{
    public function getColor(): string
    {
        return 'Red';
    }

    public function isSweet(): bool
    {
        return true;
    }
}

class Orange extends Fruit
{
    public function getColor(): string
    {
        return 'Orange';
    }

    public function hasCitrus(): bool
    {
        return true;
    }
}

class Banana extends Fruit
{
    public function getColor(): string
    {
        return 'Yellow';
    }

    public function isCurved(): bool
    {
        return true;
    }
}

class Grape extends Fruit
{
    public function getColor(): string
    {
        return 'Purple';
    }

    public function isSmall(): bool
    {
        return true;
    }
}

class Mango extends Fruit
{
    public function getColor(): string
    {
        return 'Yellow-Orange';
    }

    public function isTropical(): bool
    {
        return true;
    }
}

function describeFruit(Fruit $fruit): Mango|string
{
    if ($fruit instanceof Apple) {
        return $fruit->isSweet() ? 'Sweet apple' : 'Tart apple';
    }

    if ($fruit instanceof Orange) {
        return $fruit->hasCitrus() ? 'Citrus orange' : 'Not citrus';
    }

    if ($fruit instanceof Banana) {
        return $fruit->isCurved() ? 'Curved banana' : 'Straight banana';
    }

    if ($fruit instanceof Grape) {
        return $fruit->isSmall() ? 'Small grape' : 'Large grape';
    }

    return $fruit;
}
