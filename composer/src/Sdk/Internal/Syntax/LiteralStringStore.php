<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Syntax;

use function substr;
use function unpack;

/**
 * Binary-searchable decoded literal-string values keyed by syntax-node ID.
 *
 * @internal
 */
final class LiteralStringStore
{
    public const RECORD_SIZE = 12;

    /**
     * @param int<0, 4294967295> $count
     */
    public function __construct(
        private readonly string $records,
        private readonly string $bytes,
        private readonly int $count,
    ) {}

    /**
     * @param non-negative-int $node
     */
    public function find(int $node): ?string
    {
        $low = 0;
        $high = $this->count - 1;
        while ($low <= $high) {
            $middle = ($low + $high) >> 1;
            /** @var array{1: int<0, 4294967295>, 2: int<0, 4294967295>, 3: int<0, 4294967295>} $record */
            $record = unpack('N3', $this->records, $middle * self::RECORD_SIZE);
            if ($record[1] < $node) {
                $low = $middle + 1;
                continue;
            }
            if ($record[1] > $node) {
                $high = $middle - 1;
                continue;
            }

            return substr($this->bytes, $record[2], $record[3]);
        }

        return null;
    }
}
