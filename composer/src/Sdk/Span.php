<?php

declare(strict_types=1);

namespace Mago\Sdk;

use Mago\Sdk\Exception\InvalidArgumentException;

/**
 * A half-open byte range within a source file.
 *
 * @api
 */
final class Span
{
    /**
     * @var int<0, 4294967295>
     */
    public readonly int $start;

    /**
     * @var int<0, 4294967295>
     */
    public readonly int $end;

    public function __construct(int $start, int $end)
    {
        if ($start < 0 || $start > 4_294_967_295 || $end < 0 || $end > 4_294_967_295 || $end < $start) {
            throw new InvalidArgumentException("Invalid source span {$start}..{$end}.");
        }

        $this->start = $start;
        $this->end = $end;
    }

    public function length(): int
    {
        return $this->end - $this->start;
    }

    public function contains(self $other): bool
    {
        return $this->start <= $other->start && $other->end <= $this->end;
    }
}
