<?php
/**
 * @since 8.4
 */
function array_find(array $array, callable $callback): mixed {}
/**
 * @since 8.4
 */
function array_find_key(array $array, callable $callback): mixed {}
/**
 * @since 8.4
 */
function array_any(array $array, callable $callback): bool {}
/**
 * @since 8.4
 */
function array_all(array $array, callable $callback): bool {}
/**
 * @since 8.4
 */
function http_get_last_response_headers(): ?array {}
/**
 * @since 8.4
 */
function http_clear_last_response_headers(): void {}

/**
 * @since 8.4
 * @param array|null $options
 * @return array<int, array>
 */
function request_parse_body(?array $options = null): array {}
/**
 * @since 8.4
 */
function fpow(float $num, float $exponent): float {}

/**
 * @since 8.4
 */
enum RoundingMode implements \UnitEnum
{
    case HalfAwayFromZero;
    case HalfTowardsZero;
    case HalfEven;
    case HalfOdd;
    case TowardsZero;
    case AwayFromZero;
    case NegativeInfinity;
    case PositiveInfinity;

    public static function cases(): array {}
}
