<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Assertion;

/**
 * The relationship between an asserted value and an integer bound, string-length bound, or count.
 *
 * @api
 */
enum IntegerAssertionKind
{
    case HasExactCount;
    case HasAtLeastCount;
    case DoesNotHaveExactCount;
    case DoesNotHaveAtLeastCount;
    case IsLessThan;
    case IsLessThanOrEqual;
    case IsGreaterThan;
    case IsGreaterThanOrEqual;
    case IsLessThanFromBound;
    case IsLessThanOrEqualFromBound;
    case IsGreaterThanFromBound;
    case IsGreaterThanOrEqualFromBound;
    case StringLengthLessThan;
    case StringLengthGreaterThanOrEqual;

    public function isCount(): bool
    {
        return match ($this) {
            self::HasExactCount,
            self::HasAtLeastCount,
            self::DoesNotHaveExactCount,
            self::DoesNotHaveAtLeastCount,
                => true,
            default => false,
        };
    }
}
