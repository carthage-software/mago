<?php

declare(strict_types=1);

interface Greets
{
    public function greet(): string;
}

/**
 * @consistent-constructor
 */
class Base
{
    public function __construct(
        public int $value = 0,
    ) {}
}

/**
 * @param class-string<Base&Greets> $class
 */
function make(string $class): string
{
    $object = new $class(1);

    return $object->greet() . $object->value;
}
