<?php

/**
 * @template K
 * @template V
 *
 * @param array<K, V> $array
 * @param (callable(V, K): bool) $callback
 *
 * @return V|null
 *
 * @since 8.4
 */
function array_find(array $array, callable $callback): mixed
{
}

/**
 * @template K
 * @template V
 *
 * @param array<K, V> $array
 * @param (callable(V, K): bool) $callback
 *
 * @return K|null
 *
 * @since 8.4
 */
function array_find_key(array $array, callable $callback): mixed
{
}

/**
 * @template K
 * @template V
 *
 * @param array<K, V> $array
 * @param (callable(V, K): bool) $callback
 *
 * @since 8.4
 */
function array_any(array $array, callable $callback): bool
{
}

/**
 * @template K
 * @template V
 *
 * @param array<K, V> $array
 * @param (callable(V, K): bool) $callback
 *
 * @since 8.4
 */
function array_all(array $array, callable $callback): bool
{
}

/**
 * @return null|list<non-empty-string>
 *
 * @since 8.4
 */
function http_get_last_response_headers(): null|array
{
}

/**
 * @since 8.4
 */
function http_clear_last_response_headers(): void
{
}

/**
 * @since 8.4
 * @param array|null $options
 * @return array<int, array>
 */
function request_parse_body(null|array $options = null): array
{
}

function fpow(float $num, float $exponent): float
{
}

enum RoundingMode implements UnitEnum
{
    case HalfAwayFromZero;
    case HalfTowardsZero;
    case HalfEven;
    case HalfOdd;
    case TowardsZero;
    case AwayFromZero;
    case NegativeInfinity;
    case PositiveInfinity;
}
