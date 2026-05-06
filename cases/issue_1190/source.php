<?php

declare(strict_types=1);

final class ConstHolder
{
    const int FOO = 7;
    const int BAR = 8;
    const string NAME = 'name';
}

enum Status: int
{
    case Active = 42;
    case Inactive = 99;
}

enum Color: string
{
    case Red = 'red';
    case Blue = 'blue';
}

/**
 * @psalm-type ConstKeyedArray = array{ConstHolder::FOO: string, ConstHolder::BAR: string}
 */
class ConstKeyTest
{
    /** @return ConstKeyedArray */
    public function getConstKeyed(): array
    {
        return [7 => 'a', 8 => 'b'];
    }
}

/**
 * @psalm-type EnumKeyedArray = array{Status::Active: string, Status::Inactive: string}
 */
class EnumKeyTest
{
    /** @return EnumKeyedArray */
    public function getEnumKeyed(): array
    {
        return [42 => 'a', 99 => 'b'];
    }
}

/**
 * @psalm-type StringConstKeyedArray = array{Color::Red: int, Color::Blue: int}
 */
class StringEnumKeyTest
{
    /** @return StringConstKeyedArray */
    public function getStringEnumKeyed(): array
    {
        return ['red' => 1, 'blue' => 2];
    }
}

/**
 * @psalm-type StringConstKeyedArray2 = array{ConstHolder::NAME: int}
 */
class StringConstKeyTest
{
    /** @return StringConstKeyedArray2 */
    public function getStringConstKeyed(): array
    {
        return ['name' => 42];
    }
}

/**
 * @psalm-type MixedKeyArray = array{ConstHolder::FOO: string, Status::Active: int, 'literal': bool}
 */
class MixedKeyTest
{
    /** @return MixedKeyArray */
    public function getMixed(): array
    {
        return [7 => 'a', 42 => 1, 'literal' => true];
    }
}
