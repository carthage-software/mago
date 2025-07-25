<?php

namespace Decimal;

use ArithmeticError;
use BadMethodCallException;
use DomainException;
use Traversable;
use TypeError;

final class Decimal implements \JsonSerializable
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
     * @param Decimal|string|int $value
     *
     * @throws BadMethodCallException
     * @throws TypeError
     * @throws DomainException
     */
    public function __construct($value, int $precision = Decimal::DEFAULT_PRECISION) {}

    /**
     * @param array|Traversable $values
     *
     * @return Decimal the sum of all given values.
     *
     * @throws TypeError
     * @throws ArithmeticError
     */
    public static function sum($values, int $precision = Decimal::DEFAULT_PRECISION): Decimal
    {
    }

    /**
     * Average
     *
     * The precision of the result will be the max of all precisions that were
     * encountered during the calculation. The given precision should therefore
     * be considered the minimum precision of the result.
     *
     * This method is equivalent to adding each value individually,
     * then dividing by the number of values.
     *
     * @param array|\Traversable $values
     * @param int                $precision Minimum precision of the average.
     *
     * @return Decimal the average of all given values.
     *
     * @throws \TypeError if an unsupported type is encountered.
     * @throws \ArithmeticError if addition is undefined, eg. INF + -INF
     */
    public static function avg($values, int $precision = Decimal::DEFAULT_PRECISION): Decimal
    {
    }

    /**
     * Copy
     *
     * @param null|int $precision The precision of the return value, which defaults
     *                       to the precision of this decimal.
     *
     * @return Decimal a copy of this decimal.
     */
    public function copy(null|int $precision = null): Decimal
    {
    }

    /**
     * Add
     *
     * This method is equivalent to the `+` operator.
     *
     * The precision of the result will be the max of this decimal's precision
     * and the given value's precision, where scalar values assume the default.
     *
     * @param Decimal|string|int $value
     *
     * @return Decimal the result of adding this decimal to the given value.
     *
     * @throws \TypeError if the value is not a decimal, string or integer.
     */
    public function add($value): Decimal
    {
    }

    /**
     * Subtract
     *
     * This method is equivalent to the `-` operator.
     *
     * The precision of the result will be the max of this decimal's precision
     * and the given value's precision, where scalar values assume the default.
     *
     * @param Decimal|string|int $value
     *
     * @return Decimal the result of subtracting a given value from this decimal.
     *
     * @throws \TypeError if the value is not a decimal, string or integer.
     */
    public function sub($value): Decimal
    {
    }

    /**
     * Multiply
     *
     * This method is equivalent to the `*` operator.
     *
     * The precision of the result will be the max of this decimal's precision
     * and the given value's precision, where scalar values assume the default.
     *
     * @param Decimal|string|int $value
     *
     * @return Decimal the result of multiplying this decimal by the given value.
     *
     * @throws \TypeError if the given value is not a decimal, string or integer.
     */
    public function mul($value): Decimal
    {
    }

    /**
     * Divide
     *
     * This method is equivalent to the `/` operator.
     *
     * The precision of the result will be the max of this decimal's precision
     * and the given value's precision, where scalar values assume the default.
     *
     * @param Decimal|string|int $value
     *
     * @return Decimal the result of dividing this decimal by the given value.
     *
     * @throws \TypeError if the value is not a decimal, string or integer.
     * @throws \DivisionByZeroError if dividing by zero.
     * @throws \ArithmeticError if division is undefined, eg. INF / -INF
     */
    public function div($value): Decimal
    {
    }

    /**
     * Modulo (integer)
     *
     * This method is equivalent to the `%` operator.
     *
     * The precision of the result will be the max of this decimal's precision
     * and the given value's precision, where scalar values assume the default.
     *
     * @see Decimal::rem for the decimal remainder.
     *
     * @param Decimal|string|int $value
     *
     * @return Decimal the remainder after dividing the integer value of this
     *                 decimal by the integer value of the given value
     *
     * @throws \TypeError if the value is not a decimal, string or integer.
     * @throws \DivisionByZeroError if the integer value of $value is zero.
     * @throws \ArithmeticError if the operation is undefined, eg. INF % -INF
     */
    public function mod($value): Decimal
    {
    }

    /**
     * Remainder
     *
     * The precision of the result will be the max of this decimal's precision
     * and the given value's precision, where scalar values assume the default.
     *
     * @param Decimal|string|int $value
     *
     * @return Decimal the remainder after dividing this decimal by a given value.
     *
     * @throws \TypeError if the value is not a decimal, string or integer.
     * @throws \DivisionByZeroError if the integer value of $value is zero.
     * @throws \ArithmeticError if the operation is undefined, eg. INF, -INF
     */
    public function rem($value): Decimal
    {
    }

    /**
     * Power
     *
     * This method is equivalent to the `**` operator.
     *
     * The precision of the result will be the max of this decimal's precision
     * and the given value's precision, where scalar values assume the default.
     *
     * @param Decimal|string|int $exponent The power to raise this decimal to.
     *
     * @return Decimal the result of raising this decimal to a given power.
     *
     * @throws \TypeError if the exponent is not a decimal, string or integer.
     */
    public function pow($exponent): Decimal
    {
    }

    /**
     * Natural logarithm
     *
     * This method is equivalent in function to PHP's `log`.
     *
     * @return Decimal the natural logarithm of this decimal (log base e),
     *                 with the same precision as this decimal.
     */
    public function ln(): Decimal
    {
    }

    /**
     * Exponent
     *
     * @return Decimal the exponent of this decimal, ie. e to the power of this,
     *                 with the same precision as this decimal.
     */
    public function exp(): Decimal
    {
    }

    /**
     * Base-10 logarithm
     *
     * @return Decimal the base-10 logarithm of this decimal, with the same
     *                 precision as this decimal.
     */
    public function log10(): Decimal
    {
    }

    /**
     * Square root
     *
     * @return Decimal the square root of this decimal, with the same precision
     *                 as this decimal.
     */
    public function sqrt(): Decimal
    {
    }

    /**
     * Floor
     *
     * @return Decimal the closest integer towards negative infinity.
     */
    public function floor(): Decimal
    {
    }

    /**
     * Ceiling
     *
     * @return Decimal the closest integer towards positive infinity.
     */
    public function ceil(): Decimal
    {
    }

    /**
     * Truncate
     *
     * @return Decimal the integer value of this decimal.
     */
    public function truncate(): Decimal
    {
    }

    /**
     * Round
     *
     * @param int $places The number of places behind the decimal to round to.
     * @param int $mode   The rounding mode, which are constants of Decimal.
     *
     * @return Decimal the value of this decimal with the same precision,
     *                 rounded according to the specified number of decimal
     *                 places and rounding mode
     *
     * @throws \InvalidArgumentException if the rounding mode is not supported.
     */
    public function round(int $places = 0, int $mode = Decimal::DEFAULT_ROUNDING): Decimal
    {
    }

    /**
     * Decimal point shift.
     *
     * @param int $places The number of places to shift the decimal point by.
     *                    A positive shift moves the decimal point to the right,
     *                    a negative shift moves the decimal point to the left.
     *
     * @return Decimal A copy of this decimal with its decimal place shifted.
     */
    public function shift(int $places): Decimal
    {
    }

    /**
     * Trims trailing zeroes.
     *
     * @return Decimal A copy of this decimal without trailing zeroes.
     */
    public function trim(): Decimal
    {
    }

    /**
     * Precision
     *
     * @return int the precision of this decimal.
     */
    public function precision(): int
    {
    }

    /**
     * Signum
     *
     * @return int 0 if zero, -1 if negative, or 1 if positive.
     */
    public function signum(): int
    {
    }

    /**
     * Parity (integer)
     *
     * @return int 0 if the integer value of this decimal is even, 1 if odd.
     *             Special numbers like NAN and INF will return 1.
     */
    public function parity(): int
    {
    }

    /**
     * Absolute
     *
     * @return Decimal the absolute (positive) value of this decimal.
     */
    public function abs(): Decimal
    {
    }

    /**
     * Negate
     *
     * @return Decimal the same value as this decimal, but the sign inverted.
     */
    public function negate(): Decimal
    {
    }

    /**
     * @return bool TRUE if this decimal is an integer and even, FALSE otherwise.
     */
    public function isEven(): bool
    {
    }

    /**
     * @return bool TRUE if this decimal is an integer and odd, FALSE otherwise.
     */
    public function isOdd(): bool
    {
    }

    /**
     * @return bool TRUE if this decimal is positive, FALSE otherwise.
     */
    public function isPositive(): bool
    {
    }

    /**
     * @return bool TRUE if this decimal is negative, FALSE otherwise.
     */
    public function isNegative(): bool
    {
    }

    /**
     * @return bool TRUE if this decimal is not a defined number.
     */
    public function isNaN(): bool
    {
    }

    /**
     * @return bool TRUE if this decimal represents infinity, FALSE otherwise.
     */
    public function isInf(): bool
    {
    }

    /**
     * @return bool TRUE if this decimal is an integer, ie. does not have
     *              significant figures behind the decimal point, otherwise FALSE.
     */
    public function isInteger(): bool
    {
    }

    /**
     * @return bool TRUE if this decimal is either positive or negative zero.
     */
    public function isZero(): bool
    {
    }

    /**
     * @param int  $places   The number of places behind the decimal point.
     * @param bool $commas   TRUE if thousands should be separated by a comma.
     * @param int  $rounding
     *
     * @return string the value of this decimal formatted to a fixed number of
     *                decimal places, optionally with thousands comma-separated,
     *                using a given rounding mode.
     */
    public function toFixed(int $places = 0, bool $commas = false, int $rounding = Decimal::DEFAULT_ROUNDING): string
    {
    }

    /**
     * String representation.
     *
     * This method is equivalent to a cast to string.
     *
     * This method should not be used as a canonical representation of this
     * decimal, because values can be represented in more than one way. However,
     * this method does guarantee that a decimal instantiated by its output with
     * the same precision will be exactly equal to this decimal.
     *
     * @return string the value of this decimal represented exactly, in either
     *                fixed or scientific form, depending on the value.
     */
    public function toString(): string
    {
    }

    /**
     * Integer representation.
     *
     * This method is equivalent to a cast to int.
     *
     * @return int the integer value of this decimal.
     *
     * @throws \OverflowException if the value is greater than PHP_INT_MAX.
     */
    public function toInt(): int
    {
    }

    /**
     * Binary floating point representation.
     *
     * This method is equivalent to a cast to float, and is not affected by the
     * 'precision' INI setting.
     *
     * @return float the native PHP floating point value of this decimal.
     *
     * @throws \OverflowException  if the value is greater than PHP_FLOAT_MAX.
     * @throws \UnderflowException if the value is smaller than PHP_FLOAT_MIN.
     */
    public function toFloat(): float
    {
    }

    /**
     * Equality
     *
     * This method is equivalent to the `==` operator.
     *
     * @param mixed $other
     *
     * @return bool TRUE if this decimal is considered equal to the given value.
     *              Equal decimal values tie-break on precision.
     */
    public function equals($other): bool
    {
    }

    /**
     * Ordering
     *
     * This method is equivalent to the `<=>` operator.
     *
     * @param mixed $other
     *
     * @return int  0 if this decimal is considered is equal to $other,
     *             -1 if this decimal should be placed before $other,
     *              1 if this decimal should be placed after $other.
     */
    public function compareTo($other): int
    {
    }

    /**
     * String representation.
     *
     * This method is equivalent to a cast to string, as well as `toString`.
     *
     * @return string the value of this decimal represented exactly, in either
     *                fixed or scientific form, depending on the value.
     */
    public function __toString(): string
    {
    }

    /**
     * JSON
     *
     * This method is only here to honour the interface, and is equivalent to
     * `toString`. JSON does not have a decimal type so all decimals are encoded
     * as strings in the same format as `toString`.
     *
     * @return string
     */
    public function jsonSerialize()
    {
    }
}
