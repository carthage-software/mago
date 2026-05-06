<?php

declare(strict_types=1);

final class A {}

final class B {}

/**
 * @param array<int, A|B> $abIn
 * @return array<A>
 */
function filter_to_a(array $abIn): array
{
    return array_filter($abIn, fn(A|B $ab): bool => $ab instanceof A);
}

/**
 * @param array<int, A|B> $abIn
 * @return array<B>
 */
function filter_excluding_a(array $abIn): array
{
    return array_filter($abIn, fn(A|B $ab): bool => !$ab instanceof A);
}

/**
 * @param array<int, int|string> $values
 * @return array<int>
 */
function only_ints(array $values): array
{
    return array_filter($values, fn(int|string $v): bool => is_int($v));
}

/**
 * @param array<int, int|string> $values
 * @return array<string>
 */
function only_strings(array $values): array
{
    return array_filter($values, fn(int|string $v): bool => is_string($v));
}

/**
 * @param array<int, ?A> $items
 * @return array<A>
 */
function only_present(array $items): array
{
    return array_filter($items, fn(?A $a): bool => $a !== null);
}

/**
 * @param array<int, ?A> $items
 * @return array<A>
 */
function only_present_block(array $items): array
{
    return array_filter($items, function (?A $a): bool {
        return $a !== null;
    });
}

function is_a_instance(A|B $ab): bool
{
    return $ab instanceof A;
}

class Predicate
{
    public function isInt(mixed $v): bool
    {
        return is_int($v);
    }

    public function isPresent(?A $a): bool
    {
        return $a !== null;
    }
}

/**
 * @param array<int, A|B> $abIn
 * @return array<A>
 */
function filter_via_function(array $abIn): array
{
    return array_filter($abIn, is_a_instance(...));
}

/**
 * @param array<int, int|string> $values
 * @return array<int>
 */
function filter_via_method(array $values): array
{
    $p = new Predicate();
    return array_filter($values, $p->isInt(...));
}

/**
 * @param array<int, ?A> $items
 * @return array<A>
 */
function filter_via_method_present(array $items): array
{
    $p = new Predicate();
    return array_filter($items, $p->isPresent(...));
}
