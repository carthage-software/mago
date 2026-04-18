<?php

final class TypeIdentifier
{
    public const NULL = 1;
    public const ARRAY = 2;
    public const INT = 3;
    public const STRING = 4;
    public const FLOAT = 5;
    public const TRUE = 6;
    public const FALSE = 7;
    public const MIXED = 8;
    public const CALLABLE = 9;
    public const ITERABLE = 10;
    public const RESOURCE = 11;
    public const BOOL = 12;
    public const OBJECT = 13;
}

/**
 * @return TypeIdentifier::NULL|TypeIdentifier::ARRAY|TypeIdentifier::INT
 */
function pick(): int
{
    return TypeIdentifier::NULL;
}

/**
 * @param TypeIdentifier::ITERABLE|TypeIdentifier::RESOURCE $id
 */
function takes_builtin(int $id): void {}

function test_class_constant_reserved_name(): void
{
    $picked = pick();
    takes_builtin(TypeIdentifier::ITERABLE);
    takes_builtin(TypeIdentifier::RESOURCE);
    echo $picked;
}
