<?php

declare(strict_types=1);

namespace Mago\Sdk\Testing;

use Mago\Sdk\Exception\InvalidArgumentException;
use Mago\Sdk\Internal\Syntax\NodeStore;
use Mago\Sdk\Internal\Syntax\ResolvedNameStore;
use Mago\Sdk\Internal\Syntax\TriviaStore;
use Mago\Sdk\PHPVersion;
use Mago\Sdk\Syntax\NodeKind;
use Mago\Sdk\Syntax\SourceFile;

use function array_key_exists;
use function count;
use function pack;
use function strlen;

/**
 * Builds synthetic syntax snapshots for extension tests.
 *
 * The builder packs the same fixed-width node tables Mago sends over the
 * extension protocol, so rules under test observe identical traversal,
 * parenting, text, and resolved-name behavior without a worker or host.
 *
 * @api
 * @mago-expect lint:cyclomatic-complexity
 */
final class SourceFileBuilder
{
    private const NO_NODE = 4_294_967_295;

    /**
     * @var list<array{kind: NodeKind, start: int<0, 4294967295>, end: int<0, 4294967295>, parent: int<0, 4294967295>|null}>
     */
    private array $nodes = [];

    /**
     * @var list<int<0, 4294967295>>
     */
    private array $targetNodeIds = [];

    /**
     * @var list<array{start: int<0, 4294967295>, end: int<0, 4294967295>, name: string, imported: bool}>
     */
    private array $resolvedNames = [];

    private PHPVersion $phpVersion;

    public function __construct(
        private readonly string $path = 'test.php',
        private readonly string $contents = '',
        ?PHPVersion $phpVersion = null,
    ) {
        $this->phpVersion = $phpVersion ?? PHPVersion::fromParts(8, 4);
    }

    /**
     * Adds a node and returns its identifier. Children keep insertion order.
     *
     * @param int<0, 4294967295> $start
     * @param int<0, 4294967295> $end
     * @param int<0, 4294967295>|null $parent
     *
     * @return int<0, 4294967295>
     */
    public function addNode(NodeKind $kind, int $start, int $end, ?int $parent = null): int
    {
        if ($end < $start) {
            throw new InvalidArgumentException("Invalid test node span {$start}..{$end}.");
        }

        if ($parent !== null && !array_key_exists($parent, $this->nodes)) {
            throw new InvalidArgumentException("Test node parent {$parent} does not exist.");
        }

        $this->nodes[] = ['kind' => $kind, 'start' => $start, 'end' => $end, 'parent' => $parent];

        return count($this->nodes) - 1;
    }

    /**
     * Marks a node as a lint target visited by `RuleRunner`.
     *
     * @param int<0, 4294967295> $id
     */
    public function addTarget(int $id): void
    {
        if (!array_key_exists($id, $this->nodes)) {
            throw new InvalidArgumentException("Test target node {$id} does not exist.");
        }

        $this->targetNodeIds[] = $id;
    }

    /**
     * Attaches a resolved name looked up by its start offset.
     *
     * @param int<0, 4294967295> $start
     * @param int<0, 4294967295> $end
     */
    public function addResolvedName(int $start, int $end, string $name, bool $imported = false): void
    {
        if ($end < $start || $name === '') {
            throw new InvalidArgumentException('A test resolved name needs a valid span and a non-empty name.');
        }

        $this->resolvedNames[] = ['start' => $start, 'end' => $end, 'name' => $name, 'imported' => $imported];
    }

    public function build(): SourceFile
    {
        $kinds = NodeKind::cases();
        $kindIndices = [];
        foreach ($kinds as $index => $kind) {
            $kindIndices[$kind->value] = $index;
        }

        $count = count($this->nodes);
        $firstChild = [];
        $nextSibling = [];
        $lastChild = [];
        for ($id = 0; $id < $count; ++$id) {
            $firstChild[$id] = self::NO_NODE;
            $nextSibling[$id] = self::NO_NODE;
        }

        foreach ($this->nodes as $id => $node) {
            $parent = $node['parent'];
            if ($parent === null) {
                continue;
            }

            if ($firstChild[$parent] === self::NO_NODE) {
                $firstChild[$parent] = $id;
                $lastChild[$parent] = $id;
                continue;
            }

            $nextSibling[$lastChild[$parent]] = $id;
            $lastChild[$parent] = $id;
        }

        $records = '';
        foreach ($this->nodes as $id => $node) {
            $records .= pack(
                'CNNNNN',
                $kindIndices[$node['kind']->value],
                $node['start'],
                $node['end'],
                $node['parent'] ?? self::NO_NODE,
                $firstChild[$id],
                $nextSibling[$id],
            );
        }

        $starts = '';
        $nameRecords = '';
        $bytes = '';
        foreach ($this->resolvedNames as $resolvedName) {
            $starts .= pack('N', $resolvedName['start']);
            $nameRecords .= pack(
                'NNNC',
                $resolvedName['end'],
                strlen($bytes),
                strlen($resolvedName['name']),
                $resolvedName['imported'] ? 1 : 0,
            );
            $bytes .= $resolvedName['name'];
        }

        return new SourceFile(
            $this->phpVersion,
            $this->path,
            $this->contents,
            $this->targetNodeIds,
            new NodeStore($kinds, $records, $count),
            new ResolvedNameStore($starts, $nameRecords, $bytes, count($this->resolvedNames)),
            new TriviaStore('', 0),
            null,
        );
    }
}
