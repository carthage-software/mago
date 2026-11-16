<?php

declare(strict_types=1);

/**
 * @template T of object
 *
 * @param T $object
 *
 * @return class-string<T>
 */
function class_of(object $object): string
{
    return $object::class;
}

/**
 * @template T
 *
 * @param T $value
 *
 * @return T
 */
function identity(mixed $value): mixed
{
    return $value;
}

/**
 * @param list<object> $objects
 *
 * @return list<string>
 */
function class_names(array $objects): array
{
    return array_map(get_class(...), $objects);
}

/**
 * @param list<object> $objects
 *
 * @return list<string>
 */
function user_class_names(array $objects): array
{
    return array_map(class_of(...), $objects);
}

/**
 * @param callable(object): class-string $callback
 */
function take_class_name_callback(callable $callback): void
{
    $callback(new stdClass());
}

/**
 * @param callable(int): string $callback
 */
function take_int_to_string_callback(callable $callback): void
{
    $callback(1);
}

function pass_callbacks(): void
{
    take_class_name_callback(get_class(...));
    take_class_name_callback(class_of(...));
    // @mago-expect analysis:invalid-argument
    take_int_to_string_callback(identity(...));
}
