<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Syntax;

use Mago\Sdk\Exception\InvalidArgumentException;
use Mago\Sdk\Span;
use Mago\Sdk\Syntax\Node;
use Mago\Sdk\Syntax\NodeKind;

use function ord;
use function unpack;

/**
 * Lazily materializes public nodes from Mago's fixed-width node table.
 *
 * @internal
 * @mago-expect lint:cyclomatic-complexity
 */
final class NodeStore
{
    public const RECORD_SIZE = 21;

    private const NO_NODE = 4_294_967_295;

    /**
     * @var array<int<0, max>, Node>
     */
    private array $nodes = [];

    /**
     * @param list<NodeKind> $kinds
     * @param int<0, 4294967295> $nodeCount
     */
    public function __construct(
        private readonly array $kinds,
        private readonly string $records,
        private readonly int $nodeCount,
    ) {}

    public function get(int $id): Node
    {
        if ($id < 0 || $id >= $this->nodeCount) {
            throw new InvalidArgumentException("The source snapshot has no node with identifier {$id}.");
        }

        return $this->nodes[$id] ??= $this->materialize($id);
    }

    /**
     * @param array<int, int<0, 4294967295>> $ids
     *
     * @return list<Node>
     */
    public function getMany(array $ids): array
    {
        $nodes = [];
        foreach ($ids as $id) {
            $node = $this->nodes[$id] ?? null;
            if ($node === null) {
                $offset = $id * self::RECORD_SIZE;
                /** @var array{1: int<0, 4294967295>, 2: int<0, 4294967295>, 3: int<0, 4294967295>} $record */
                $record = unpack('N3', $this->records, $offset + 1);
                $parent = $record[3];
                $node = new Node(
                    $id,
                    $this->kinds[ord($this->records[$offset])],
                    new Span($record[1], $record[2]),
                    $parent === self::NO_NODE ? null : $parent,
                );
                $this->nodes[$id] = $node;
            }

            $nodes[] = $node;
        }

        return $nodes;
    }

    /**
     * @return list<Node>
     * @mago-expect lint:no-multi-assignments
     */
    public function getAll(?NodeKind $kind = null): array
    {
        $nodes = [];
        for ($id = 0, $offset = 0; $id < $this->nodeCount; ++$id, $offset += self::RECORD_SIZE) {
            if ($kind !== null && $this->kinds[ord($this->records[$offset])] !== $kind) {
                continue;
            }

            $nodes[] = $this->nodes[$id] ??= $this->materialize($id);
        }

        return $nodes;
    }

    /**
     * @return list<Node>
     * @mago-expect lint:no-multi-assignments
     */
    public function getChildren(Node $node): array
    {
        /** @var array{1: int<0, 4294967295>} $decoded */
        $decoded = unpack('N', $this->records, ($node->id * self::RECORD_SIZE) + 13);
        $children = [];
        $child = $decoded[1];
        while ($child !== self::NO_NODE) {
            /** @var array{1: int<0, 4294967295>, 2: int<0, 4294967295>, 3: int<0, 4294967295>} $childRecord */
            $childRecord = unpack('N3', $this->records, ($child * self::RECORD_SIZE) + 9);
            $children[] = $this->nodes[$child] ??= $this->materialize($child);
            $child = $childRecord[3];
        }

        return $children;
    }

    /**
     * @param non-negative-int $id
     */
    private function materialize(int $id): Node
    {
        $offset = $id * self::RECORD_SIZE;
        /** @var array{1: int<0, 4294967295>, 2: int<0, 4294967295>, 3: int<0, 4294967295>} $record */
        $record = unpack('N3', $this->records, $offset + 1);
        $kind = $this->kinds[ord($this->records[$offset])];
        $start = $record[1];
        $end = $record[2];
        $parent = $record[3];

        return new Node($id, $kind, new Span($start, $end), $parent === self::NO_NODE ? null : $parent);
    }
}
