<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Syntax;

use Mago\Sdk\Span;
use Mago\Sdk\Syntax\ResolvedName;

use function array_flip;
use function ord;
use function substr;
use function unpack;

/**
 * Binary-searchable packed resolved-name metadata with lazy objects.
 *
 * @internal
 */
final class ResolvedNameStore
{
    public const START_SIZE = 4;
    public const RECORD_SIZE = 13;

    /**
     * @var array<int<0, max>, ResolvedName>
     */
    private array $names = [];

    /**
     * @var array<int<0, 4294967295>, int<0, max>>
     */
    private ?array $indicesByStart = null;

    /**
     * @param int<0, 4294967295> $nameCount
     */
    public function __construct(
        private readonly string $starts,
        private readonly string $records,
        private readonly string $bytes,
        private readonly int $nameCount,
    ) {}

    /**
     * @param int<0, 4294967295> $start
     */
    public function find(int $start): ?ResolvedName
    {
        $index = ($this->indicesByStart ??= $this->index())[$start] ?? null;

        return $index === null ? null : $this->get($index, $start);
    }

    /**
     * @return array<int<0, 4294967295>, int<0, max>>
     */
    private function index(): array
    {
        /** @var array<int<0, 4294967295>, int<1, max>> */
        return array_flip(unpack('N*', $this->starts));
    }

    /**
     * @return list<ResolvedName>
     */
    public function all(?Span $within = null): array
    {
        $names = [];
        /** @var array<int<1, max>, int<0, 4294967295>> $starts */
        $starts = unpack('N*', $this->starts);
        for ($index = 1; $index <= $this->nameCount; ++$index) {
            $name = $this->get($index, $starts[$index]);
            if ($within === null || $within->contains($name->span)) {
                $names[] = $name;
            }
        }

        return $names;
    }

    /**
     * @param int<0, max> $index
     */
    private function get(int $index, int $start): ResolvedName
    {
        return $this->names[$index] ??= $this->materialize($index, $start);
    }

    /**
     * @param int<0, max> $index
     */
    private function materialize(int $index, int $start): ResolvedName
    {
        $recordOffset = ($index - 1) * self::RECORD_SIZE;
        /** @var array{1: int<0, 4294967295>, 2: int<0, 4294967295>, 3: int<0, 4294967295>} $record */
        $record = unpack('N3', $this->records, $recordOffset);

        return new ResolvedName(
            new Span($start, $record[1]),
            substr($this->bytes, $record[2], $record[3]),
            ord($this->records[$recordOffset + 12]) === 1,
        );
    }
}
