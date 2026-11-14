<?php

declare(strict_types=1);

/**
 * @template T
 * @param T $value
 * @return (T is null ? never : T)
 * @throws LogicException
 */
function conditional_part_0_not_null(mixed $value): mixed
{
    return $value ?? throw new LogicException();
}

/**
 * @template T
 * @param T $value
 * @return (T is empty ? T : never)
 * @throws LogicException
 */
function conditional_part_0_only_falsy(mixed $value): mixed
{
    return $value ? throw new LogicException() : $value;
}

/**
 * @template T
 * @param T $value
 * @return (T is string ? T : int)
 */
function conditional_part_0_keep_string(mixed $value): mixed
{
    return is_string($value) ? $value : 0;
}

/** @throws LogicException */
function conditional_part_0_a(?int $x): int
{
    return conditional_part_0_not_null($x);
}

/** @throws LogicException */
function conditional_part_0_b(bool $x): false
{
    return conditional_part_0_only_falsy($x);
}

function conditional_part_0_c(string|float $x): string|int
{
    return conditional_part_0_keep_string($x);
}

/**
 * @template T
 * @param T $value
 * @return (T is empty ? null : T)
 */
function conditional_part_1_null_if_empty(mixed $value): mixed
{
    return $value ?: null;
}

/**
 * @param non-falsy-string $value
 * @return non-falsy-string
 */
function conditional_part_1_a(string $value): string
{
    return conditional_part_1_null_if_empty($value);
}

/**
 * @param non-empty-list<int> $value
 * @return non-empty-list<int>
 */
function conditional_part_1_b(array $value): array
{
    return conditional_part_1_null_if_empty($value);
}

/**
 * @param array{a: int} $value
 * @return array{a: int}
 */
function conditional_part_1_c(array $value): array
{
    return conditional_part_1_null_if_empty($value);
}
