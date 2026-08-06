<?php

declare(strict_types=1);

final readonly class Issue2175C
{
    public function __construct(
        public null|int $p = null,
    ) {}
}

array_filter(get_object_vars(new Issue2175C()), static fn(mixed $value): bool => $value !== null);

/**
 * @param array<string, int|null> $a
 */
function issue_2175_arrow(array $a): void
{
    array_map(static fn(mixed $value): bool => $value !== null, $a);
}

/**
 * @param array<string, int|null> $a
 */
function issue_2175_closure(array $a): void
{
    array_map(static function (mixed $value): bool {
        return $value !== null;
    }, $a);
}

/**
 * @param array<string, int|null> $a
 */
function issue_2175_declared_parameter(array $a): void
{
    array_map(static fn(null|int $value): bool => $value !== null, $a);
}

/**
 * @param array<string, list<int>> $a
 */
function issue_2175_vanilla_array_parameter(array $a): void
{
    array_map(static fn(array $value): int => count($value), $a);
}

/**
 * @param array<string, int> $a
 */
function issue_2175_non_nullable_values(array $a): void
{
    array_map(
        /**
         * @mago-expect analysis:redundant-comparison
         */
        static fn(mixed $value): bool => $value !== null,
        $a,
    );
}

/**
 * @param array<string, int|null> $a
 */
function issue_2175_nullable_reaches_body(array $a): void
{
    array_map(
        /**
         * @mago-expect analysis:nullable-return-statement
         * @mago-expect analysis:invalid-return-statement
         */
        static fn(mixed $value): int => $value,
        $a,
    );
}
