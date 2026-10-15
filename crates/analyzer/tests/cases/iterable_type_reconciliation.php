<?php

declare(strict_types=1);

use Traversable;

/**
 * @param Traversable<int, string> $_
 */
function accept_traversable(Traversable $_): void {}

/**
 * @param iterable<int, string> $it
 */
function iterable_not_array_is_traversable(iterable $it): void
{
    if (!is_array($it)) {
        accept_traversable($it);
    }
}

/**
 * @param iterable<int, string> $it
 */
function iterable_is_object_is_traversable(iterable $it): void
{
    if (is_object($it)) {
        accept_traversable($it);
    }
}

/**
 * @param iterable<int, string> $it
 */
function iterable_bare_stays_iterable(iterable $it): void
{
    accept_traversable($it); // @mago-expect analysis:possibly-invalid-argument
}

/**
 * @assert-if-true array{special: string} $value
 */
function is_special_shape(mixed $value): bool
{
    return is_array($value) && isset($value['special']) && is_string($value['special']);
}

/**
 * @param Traversable<string, string> $_
 */
function accept_string_traversable(Traversable $_): void {}

/**
 * @param iterable<string, string> $it
 */
function iterable_not_special_shape_stays_iterable(iterable $it): void
{
    if (!is_special_shape($it)) {
        accept_string_traversable($it); // @mago-expect analysis:possibly-invalid-argument
    }
}

/**
 * @assert-if-true list<string> $value
 */
function is_string_list(mixed $value): bool
{
    return is_array($value);
}

/**
 * @param iterable<int, string> $it
 */
function iterable_not_list_stays_iterable(iterable $it): void
{
    if (!is_string_list($it)) {
        accept_traversable($it); // @mago-expect analysis:possibly-invalid-argument
    }
}

/**
 * @assert-if-true array<int, int> $value
 */
function is_int_map(mixed $value): bool
{
    return is_array($value);
}

/**
 * @param Traversable<int, mixed> $_
 */
function accept_mixed_traversable(Traversable $_): void {}

/**
 * @param iterable<int, mixed> $it
 */
function iterable_not_int_map_stays_iterable(iterable $it): void
{
    if (!is_int_map($it)) {
        accept_mixed_traversable($it); // @mago-expect analysis:possibly-invalid-argument
    }
}

/**
 * @assert-if-true array<int, string> $value
 */
function is_int_string_array(mixed $value): bool
{
    return is_array($value);
}

/**
 * @param iterable<int, string> $it
 */
function iterable_not_full_typed_array_is_traversable(iterable $it): void
{
    if (!is_int_string_array($it)) {
        accept_traversable($it);
    }
}
