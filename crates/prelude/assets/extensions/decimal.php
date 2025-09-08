<?php

namespace Decimal;

use ArithmeticError;
use BadMethodCallException;
use DivisionByZeroError;
use DomainException;
use InvalidArgumentException;
use JsonSerializable;
use OverflowException;
use Traversable;
use TypeError;
use UnderflowException;

final class Decimal implements JsonSerializable
{
    public const ROUND_UP = 0;
    public const ROUND_DOWN = 0;
    public const ROUND_CEILING = 0;
    public const ROUND_FLOOR = 0;
    public const ROUND_HALF_UP = 0;
    public const ROUND_HALF_DOWN = 0;
    public const ROUND_HALF_EVEN = 0;
    public const ROUND_HALF_ODD = 0;
    public const ROUND_TRUNCATE = 0;
    public const DEFAULT_ROUNDING = Decimal::ROUND_HALF_EVEN;
    public const DEFAULT_PRECISION = 28;
    public const MIN_PRECISION = 1;
    public const MAX_PRECISION = 0;

    /**
     * @throws BadMethodCallException
     * @throws TypeError
     * @throws DomainException
     */
    public function __construct(string|int|Decimal $value, int $precision = Decimal::DEFAULT_PRECISION) {}

    /**
     * @param array<int|string|Decimal>|Traversable<int|string|Decimal> $values
     *
     * @throws TypeError
     * @throws ArithmeticError
     */
    public static function sum(array|Traversable $values, int $precision = Decimal::DEFAULT_PRECISION): Decimal
    {
    }

    /**
     * @param array<int|string|Decimal>|Traversable<int|string|Decimal> $values
     *
     * @throws TypeError
     * @throws ArithmeticError
     */
    public static function avg(array|Traversable $values, int $precision = Decimal::DEFAULT_PRECISION): Decimal
    {
    }

    /**
     * @throws TypeError
     */
    public function add(Decimal|string|int $value): Decimal
    {
    }

    /**
     * @throws TypeError
     */
    public function sub(Decimal|string|int $value): Decimal
    {
    }

    /**
     * @throws TypeError
     */
    public function mul(Decimal|string|int $value): Decimal
    {
    }

    /**
     * @throws TypeError
     * @throws DivisionByZeroError
     * @throws ArithmeticError
     */
    public function div(Decimal|string|int $value): Decimal
    {
    }

    /**
     * @throws TypeError
     * @throws DivisionByZeroError
     * @throws ArithmeticError
     */
    public function mod(Decimal|string|int $value): Decimal
    {
    }

    /**
     * @throws TypeError
     * @throws DivisionByZeroError
     * @throws ArithmeticError
     */
    public function rem(Decimal|string|int $value): Decimal
    {
    }

    /**
     * @throws TypeError
     */
    public function pow(Decimal|string|int $exponent): Decimal
    {
    }

    public function ln(): Decimal
    {
    }

    public function exp(): Decimal
    {
    }

    public function log10(): Decimal
    {
    }

    public function sqrt(): Decimal
    {
    }

    public function floor(): Decimal
    {
    }

    public function ceil(): Decimal
    {
    }

    public function truncate(): Decimal
    {
    }

    /**
     * @throws InvalidArgumentException
     */
    public function round(int $places = 0, int $mode = Decimal::DEFAULT_ROUNDING): Decimal
    {
    }

    public function shift(int $places): Decimal
    {
    }

    public function trim(): Decimal
    {
    }

    public function precision(): int
    {
    }

    /**
     * @return -1|0|1
     */
    public function signum(): int
    {
    }

    /**
     * @return 0|1
     */
    public function parity(): int
    {
    }

    public function abs(): Decimal
    {
    }

    public function negate(): Decimal
    {
    }

    public function isEven(): bool
    {
    }

    public function isOdd(): bool
    {
    }

    public function isPositive(): bool
    {
    }

    public function isNegative(): bool
    {
    }

    public function isNaN(): bool
    {
    }

    public function isInf(): bool
    {
    }

    public function isInteger(): bool
    {
    }

    public function isZero(): bool
    {
    }

    public function toFixed(int $places = 0, bool $commas = false, int $rounding = Decimal::DEFAULT_ROUNDING): string
    {
    }

    public function toString(): string
    {
    }

    /**
     * @throws OverflowException
     */
    public function toInt(): int
    {
    }

    /**
     * @throws OverflowException
     * @throws UnderflowException
     */
    public function toFloat(): float
    {
    }

    public function equals($other): bool
    {
    }

    /**
     * @return -1|0|1
     */
    public function compareTo($other): int
    {
    }

    public function __toString(): string
    {
    }

    public function jsonSerialize(): string
    {
    }
}
