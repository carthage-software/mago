<?php declare(strict_types=1);

namespace Reproduction;

use ArrayIterator;
use Generator;
use Override;

final class InvalidRangeException extends \InvalidArgumentException
{
    /**
     * @pure
     */
    public function __construct(
        string $message,
        private readonly int $lower_bound,
        private readonly int $upper_bound,
    ) {
        parent::__construct($message);
    }

    /**
     * @pure
     */
    public static function lowerBoundIsGreaterThanUpperBound(int $lower_bound, int $upper_bound): self
    {
        return new self(
            sprintf(
                '`$lower_bound` (%d) must be less than or equal to `$upper_bound` (%d).',
                $lower_bound,
                $upper_bound,
            ),
            $lower_bound,
            $upper_bound,
        );
    }

    public function getLowerBound(): int
    {
        return $this->lower_bound;
    }

    public function getUpperBound(): int
    {
        return $this->upper_bound;
    }
}

/**
 * @psalm-immutable
 */
interface UpperBoundRangeInterface extends RangeInterface
{
    /**
     * {@inheritDoc}
     *
     * @throws InvalidRangeException If the lower bound is greater than the upper bound.
     *
     * @psalm-mutation-free
     */
    #[Override]
    public function withLowerBound(int $lower_bound): UpperBoundRangeInterface&LowerBoundRangeInterface;

    /**
     * @psalm-mutation-free
     */
    public function withoutUpperBound(): RangeInterface;

    /**
     * @psalm-mutation-free
     */
    public function getUpperBound(): int;

    /**
     * @psalm-mutation-free
     */
    public function isUpperInclusive(): bool;

    /**
     * @psalm-mutation-free
     */
    public function withUpperInclusive(bool $upper_inclusive): static;
}

/**
 * @psalm-immutable
 */
final readonly class ToRange implements UpperBoundRangeInterface
{
    private int $upperBound;
    private bool $upperInclusive;

    /**
     * @pure
     */
    public function __construct(int $upper_bound, bool $upper_inclusive = false)
    {
        $this->upperBound = $upper_bound;
        $this->upperInclusive = $upper_inclusive;
    }

    /**
     * {@inheritDoc}
     *
     * @psalm-mutation-free
     */
    #[Override]
    public function contains(int $value): bool
    {
        if ($this->upperInclusive) {
            return $value <= $this->upperBound;
        }

        return $value < $this->upperBound;
    }

    /**
     * {@inheritDoc}
     *
     * @throws InvalidRangeException If the lower bound is greater than the upper bound.
     *
     * @psalm-mutation-free
     */
    #[Override]
    public function withLowerBound(int $lower_bound): BetweenRange
    {
        return new BetweenRange($lower_bound, $this->upperBound, $this->upperInclusive);
    }

    /**
     * {@inheritDoc}
     *
     * @pure
     */
    #[Override]
    public function withoutUpperBound(): FullRange
    {
        return new FullRange();
    }

    /**
     * {@inheritDoc}
     *
     * @pure
     */
    #[Override]
    public function withUpperBound(int $upper_bound, bool $upper_inclusive): ToRange
    {
        return new ToRange($upper_bound, $upper_inclusive);
    }

    /**
     * {@inheritDoc}
     *
     * @pure
     */
    #[Override]
    public function withUpperBoundInclusive(int $upper_bound): ToRange
    {
        return new ToRange($upper_bound, true);
    }

    /**
     * {@inheritDoc}
     *
     * @pure
     */
    #[Override]
    public function withUpperBoundExclusive(int $upper_bound): ToRange
    {
        return new ToRange($upper_bound, false);
    }

    /**
     * {@inheritDoc}
     *
     * @psalm-mutation-free
     */
    #[Override]
    public function getUpperBound(): int
    {
        return $this->upperBound;
    }

    /**
     * {@inheritDoc}
     *
     * @psalm-mutation-free
     */
    #[Override]
    public function isUpperInclusive(): bool
    {
        return $this->upperInclusive;
    }

    /**
     * {@inheritDoc}
     *
     * @psalm-mutation-free
     */
    #[Override]
    public function withUpperInclusive(bool $upper_inclusive): static
    {
        return new static($this->upperBound, $upper_inclusive);
    }
}

/**
 * @psalm-immutable
 */
interface RangeInterface
{
    /**
     * @psalm-mutation-free
     */
    public function contains(int $value): bool;

    /**
     * @psalm-mutation-free
     */
    public function withLowerBound(int $lower_bound): LowerBoundRangeInterface;

    /**
     * @psalm-mutation-free
     */
    public function withUpperBound(int $upper_bound, bool $upper_inclusive): UpperBoundRangeInterface;

    /**
     * @psalm-mutation-free
     */
    public function withUpperBoundInclusive(int $upper_bound): UpperBoundRangeInterface;

    /**
     * @psalm-mutation-free
     */
    public function withUpperBoundExclusive(int $upper_bound): UpperBoundRangeInterface;
}

/**
 * @psalm-immutable
 */
interface LowerBoundRangeInterface extends RangeInterface
{
    /**
     * {@inheritDoc}
     *
     * @throws InvalidRangeException If the lower bound is greater than the upper bound.
     *
     * @psalm-mutation-free
     */
    #[Override]
    public function withUpperBound(
        int $upper_bound,
        bool $upper_inclusive,
    ): UpperBoundRangeInterface&LowerBoundRangeInterface;

    /**
     * {@inheritDoc}
     *
     * @throws InvalidRangeException If the lower bound is greater than the upper bound.
     *
     * @psalm-mutation-free
     */
    #[Override]
    public function withUpperBoundInclusive(int $upper_bound): UpperBoundRangeInterface&LowerBoundRangeInterface;

    /**
     * {@inheritDoc}
     *
     * @throws InvalidRangeException If the lower bound is greater than the upper bound.
     *
     * @psalm-mutation-free
     */
    #[Override]
    public function withUpperBoundExclusive(int $upper_bound): UpperBoundRangeInterface&LowerBoundRangeInterface;

    /**
     * @psalm-mutation-free
     */
    public function withoutLowerBound(): RangeInterface;

    /**
     * Returns the lower bound of the range.
     *
     * @psalm-mutation-free
     */
    public function getLowerBound(): int;
}

/**
 * @psalm-immutable
 */
final class FullRange implements RangeInterface
{
    /**
     * @return true
     *
     * @pure
     */
    #[Override]
    public function contains(int $value): bool
    {
        return true;
    }

    /**
     * {@inheritDoc}
     *
     * @pure
     */
    #[Override]
    public function withLowerBound(int $lower_bound): FromRange
    {
        return new FromRange($lower_bound);
    }

    /**
     * {@inheritDoc}
     *
     * @pure
     */
    #[Override]
    public function withUpperBound(int $upper_bound, bool $upper_inclusive): ToRange
    {
        return new ToRange($upper_bound, $upper_inclusive);
    }

    /**
     * {@inheritDoc}
     *
     * @pure
     */
    #[Override]
    public function withUpperBoundInclusive(int $upper_bound): ToRange
    {
        return new ToRange($upper_bound, true);
    }

    /**
     * {@inheritDoc}
     *
     * @pure
     */
    #[Override]
    public function withUpperBoundExclusive(int $upper_bound): ToRange
    {
        return new ToRange($upper_bound, false);
    }
}

/**
 * @psalm-immutable
 */
final readonly class BetweenRange implements LowerBoundRangeInterface, UpperBoundRangeInterface
{
    private int $lowerBound;
    private int $upperBound;
    private bool $upperInclusive;

    /**
     * @throws InvalidRangeException If the lower bound is greater than the upper bound.
     *
     * @psalm-mutation-free
     */
    public function __construct(int $lower_bound, int $upper_bound, bool $upper_inclusive = false)
    {
        if ($lower_bound > $upper_bound) {
            throw InvalidRangeException::lowerBoundIsGreaterThanUpperBound($lower_bound, $upper_bound);
        }

        $this->lowerBound = $lower_bound;
        $this->upperBound = $upper_bound;
        $this->upperInclusive = $upper_inclusive;
    }

    /**
     * {@inheritDoc}
     *
     * @psalm-mutation-free
     */
    #[Override]
    public function contains(int $value): bool
    {
        if ($value < $this->lowerBound) {
            return false;
        }

        if ($this->upperInclusive) {
            return $value <= $this->upperBound;
        }

        return $value < $this->upperBound;
    }

    /**
     * {@inheritDoc}
     *
     * @throws InvalidRangeException If the lower bound is greater than the upper bound.
     *
     * @psalm-mutation-free
     */
    #[Override]
    public function withUpperBound(int $upper_bound, bool $upper_inclusive): BetweenRange
    {
        return new BetweenRange($this->lowerBound, $upper_bound, $upper_inclusive);
    }

    /**
     * {@inheritDoc}
     *
     * @throws InvalidRangeException If the lower bound is greater than the upper bound.
     *
     * @psalm-mutation-free
     */
    #[Override]
    public function withUpperBoundInclusive(int $upper_bound): BetweenRange
    {
        return new BetweenRange($this->lowerBound, $upper_bound, true);
    }

    /**
     * {@inheritDoc}
     *
     * @throws InvalidRangeException If the lower bound is greater than the upper bound.
     *
     * @psalm-mutation-free
     */
    #[Override]
    public function withUpperBoundExclusive(int $upper_bound): BetweenRange
    {
        return new BetweenRange($this->lowerBound, $upper_bound, false);
    }

    /**
     * {@inheritDoc}
     *
     * @psalm-mutation-free
     */
    #[Override]
    public function withoutLowerBound(): ToRange
    {
        return new ToRange($this->upperBound, $this->upperInclusive);
    }

    /**
     * {@inheritDoc}
     *
     * @throws InvalidRangeException If the lower bound is greater than the upper bound.
     *
     * @psalm-mutation-free
     */
    #[Override]
    public function withLowerBound(int $lower_bound): BetweenRange
    {
        return new static($lower_bound, $this->upperBound, $this->upperInclusive);
    }

    /**
     * {@inheritDoc}
     *
     * @psalm-mutation-free
     */
    #[Override]
    public function withoutUpperBound(): FromRange
    {
        return new FromRange($this->lowerBound);
    }

    /**
     * {@inheritDoc}
     *
     * @psalm-mutation-free
     */
    #[Override]
    public function getUpperBound(): int
    {
        return $this->upperBound;
    }

    /**
     * {@inheritDoc}
     *
     * @psalm-mutation-free
     */
    #[Override]
    public function isUpperInclusive(): bool
    {
        return $this->upperInclusive;
    }

    /**
     * {@inheritDoc}
     *
     * @throws InvalidRangeException If the lower bound is greater than the upper bound.
     *
     * @psalm-mutation-free
     */
    #[Override]
    public function withUpperInclusive(bool $upper_inclusive): static
    {
        return new static($this->lowerBound, $this->upperBound, $upper_inclusive);
    }

    /**
     * {@inheritDoc}
     *
     * @psalm-mutation-free
     */
    #[Override]
    public function getLowerBound(): int
    {
        return $this->lowerBound;
    }
}

/**
 * @psalm-immutable
 */
final readonly class FromRange implements LowerBoundRangeInterface
{
    private int $lowerBound;

    /**
     * @psalm-mutation-free
     */
    public function __construct(int $lower_bound)
    {
        $this->lowerBound = $lower_bound;
    }

    /**
     * {@inheritDoc}
     *
     * @psalm-mutation-free
     */
    #[Override]
    public function contains(int $value): bool
    {
        return $value >= $this->lowerBound;
    }

    /**
     * {@inheritDoc}
     *
     * @pure
     */
    #[Override]
    public function withLowerBound(int $lower_bound): FromRange
    {
        return new FromRange($lower_bound);
    }

    /**
     * {@inheritDoc}
     *
     * @throws InvalidRangeException If the lower bound is greater than the upper bound.
     *
     * @psalm-mutation-free
     */
    #[Override]
    public function withUpperBound(int $upper_bound, bool $upper_inclusive): BetweenRange
    {
        return new BetweenRange($this->lowerBound, $upper_bound, $upper_inclusive);
    }

    /**
     * {@inheritDoc}
     *
     * @throws InvalidRangeException If the lower bound is greater than the upper bound.
     *
     * @psalm-mutation-free
     */
    #[Override]
    public function withUpperBoundInclusive(int $upper_bound): BetweenRange
    {
        return new BetweenRange($this->lowerBound, $upper_bound, true);
    }

    /**
     * {@inheritDoc}
     *
     * @throws InvalidRangeException If the lower bound is greater than the upper bound.
     *
     * @psalm-mutation-free
     */
    #[Override]
    public function withUpperBoundExclusive(int $upper_bound): BetweenRange
    {
        return new BetweenRange($this->lowerBound, $upper_bound, false);
    }

    /**
     * {@inheritDoc}
     *
     * @pure
     */
    #[Override]
    public function withoutLowerBound(): FullRange
    {
        return new FullRange();
    }

    /**
     * {@inheritDoc}
     *
     * @psalm-mutation-free
     */
    #[Override]
    public function getLowerBound(): int
    {
        return $this->lowerBound;
    }
}

/**
 * @pure
 */
function from(int $lower_bound): FromRange
{
    return new FromRange($lower_bound);
}

try {
    $from = from(0);

    $between = $from->withUpperBoundInclusive(10);
    // Now a BetweenRange: 0..=10

    // Remove the lower bound
    $to = $between->withoutLowerBound();
    // Now a ToRange: ..=10

    // Add a lower bound to a ToRange
    $_between = $to->withLowerBound(5);
} catch (InvalidRangeException) {
    // ignore
}
