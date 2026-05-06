<?php

declare(strict_types=1);

enum Status: string
{
    case ACTIVE = 'active';
    case INACTIVE = 'inactive';
}

/**
 * @template T of UnitEnum
 * @param class-string<T> $class
 */
function use_some_enum(string $class): void
{
    use_class_string($class);
    use_enum_string($class);
}

/**
 * @param class-string $class
 */
function use_class_string(string $class): void
{
    echo 'the class is ' . $class;
}

/**
 * @param enum-string $class
 */
function use_enum_string(string $class): void
{
    echo 'the enum is ' . $class;
}

/**
 * @template TEnum of UnitEnum
 * @param class-string<TEnum> $type
 * @throws InvalidArgumentException if the enum does not exist
 */
function process_enum(string $type): void
{
    if (!enum_exists($type)) {
        throw new InvalidArgumentException("Enum of type \"{$type}\" not found");
    }

    // $type is class-string<TEnum> where TEnum extends UnitEnum
    // type_enum accepts class-string<T> where T extends UnitEnum
    // These should be compatible
    use_some_enum($type);
    use_enum_string($type);
    use_class_string($type);
}
