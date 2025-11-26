<?php

declare(strict_types=1);

namespace Types;

use DateInterval;
use DateTimeImmutable;
use DateTimeInterface;
use DOMElement;
use Exception;
use Override;
use Throwable;

use function is_string;

/**
 * @template-covariant T
 */
interface Type
{
    /**
     * @return T
     * @throws Exception
     * @phpstan-assert T $value
     */
    public function assert(mixed $value): mixed;

    /**
     * @return T
     * @throws Exception
     */
    public function cast(mixed $value): mixed;

    /**
     * @phpstan-assert-if-true T $value
     */
    public function isValid(mixed $value): bool;
}

/**
 * @implements Type<DateInterval>
 */
final readonly class TimeType implements Type
{
    #[Override]
    public function assert(mixed $value): DateInterval
    {
        if ($this->isValid($value)) {
            return $value;
        }

        throw new Exception();
    }

    #[Override]
    public function cast(mixed $value): DateInterval
    {
        if ($this->isValid($value)) {
            return $value;
        }

        if ($value instanceof DateTimeInterface) {
            return $value->diff(new DateTimeImmutable($value->format('Y-m-d')), true);
        }

        if ($value instanceof DOMElement) {
            $value = $value->nodeValue;
        }

        try {
            if (is_string($value)) {
                return new DateInterval($value);
            }
        } catch (Throwable) {
            throw new Exception();
        }

        throw new Exception();
    }

    #[Override]
    public function isValid(mixed $value): bool
    {
        return $value instanceof DateInterval;
    }
}
