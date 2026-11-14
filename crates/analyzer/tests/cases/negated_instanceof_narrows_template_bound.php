<?php

declare(strict_types=1);

/**
 * @template T of DateTimeInterface|string
 *
 * @param T $value
 */
function take_string(DateTimeInterface|string $value): string
{
    if ($value instanceof DateTimeInterface) {
        return '';
    }

    return strtoupper($value);
}

final class Dog
{
    public function bark(): string
    {
        return 'woof';
    }
}

final class Cat
{
    public function meow(): string
    {
        return 'meow';
    }
}

/**
 * @template T of Dog|Cat
 *
 * @param T $animal
 */
function speak(Dog|Cat $animal): string
{
    if ($animal instanceof Dog) {
        return $animal->bark();
    }

    return $animal->meow();
}
