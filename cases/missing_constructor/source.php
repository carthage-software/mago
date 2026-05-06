<?php

class MissingConstructorTyped
{
    public string $name;
    public int $age;
}

// OK - has constructor
class HasConstructor
{
    public string $name;

    public function __construct()
    {
        $this->name = 'test';
    }
}

// OK - has defaults
class HasDefaults
{
    public string $name = 'default';
    public int $age = 0;
}

class NullableProperties
{
    public ?string $name;
    public ?int $age;
}

// OK - promoted properties
class PromotedProperties
{
    public function __construct(
        public string $name,
        public int $age,
    ) {}
}

// OK - abstract class doesn't need constructor
abstract class AbstractClass
{
    public string $name;
}

class ConcreteChildNoConstructor extends AbstractClass {}

// OK - no type hint means uninitialized is valid
class NoTypeHint
{
    public $name;
}

// OK - static properties are not constructor-initialized
class StaticProperties
{
    public static string $name = '';
}
