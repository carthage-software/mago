<?php

declare(strict_types=1);

namespace Mago\Sdk\Syntax;

use Mago\Sdk\Exception\InvalidArgumentException;
use Mago\Sdk\Internal\Syntax\NodeStore;
use Mago\Sdk\Internal\Syntax\ResolvedNameStore;
use Mago\Sdk\Internal\Syntax\TriviaStore;
use Mago\Sdk\PHPVersion;
use Mago\Sdk\Span;

use function strlen;
use function substr;

/**
 * An immutable syntax view of the exact source analyzed by Mago.
 *
 * @api
 */
final class SourceFile
{
    /**
     * @var array<int, int<0, 4294967295>>
     */
    private readonly array $targetNodeIds;

    /**
     * @var null|list<Node>
     */
    private ?array $targetNodes = null;

    /**
     * @param array<int, int<0, 4294967295>> $targetNodeIds
     * @internal
     * @mago-expect lint:excessive-parameter-list
     */
    public function __construct(
        public readonly PHPVersion $phpVersion,
        public readonly string $path,
        public readonly string $contents,
        array $targetNodeIds,
        private readonly NodeStore $nodes,
        private readonly ResolvedNameStore $resolvedNames,
        private readonly TriviaStore $trivia,
    ) {
        $this->targetNodeIds = $targetNodeIds;
    }

    /**
     * @return list<Node>
     */
    public function getTargetNodes(): array
    {
        if ($this->targetNodes !== null) {
            return $this->targetNodes;
        }

        return $this->targetNodes = $this->nodes->getMany($this->targetNodeIds);
    }

    /**
     * Returns every available node, optionally restricted to one kind.
     *
     * Linter snapshots contain only the subtrees selected by active rules;
     * analyzer snapshots contain the complete syntax tree.
     *
     * @return list<Node>
     */
    public function getNodes(?NodeKind $kind = null): array
    {
        return $this->nodes->getAll($kind);
    }

    /**
     * @param non-negative-int $id
     */
    public function getNode(int $id): Node
    {
        return $this->nodes->get($id);
    }

    public function getParent(Node $node): ?Node
    {
        return $node->parentId === null ? null : $this->getNode($node->parentId);
    }

    /**
     * @return list<Node>
     */
    public function getChildren(Node $node): array
    {
        return $this->nodes->getChildren($node);
    }

    public function getText(Node|Span $selection): string
    {
        $span = $selection instanceof Node ? $selection->span : $selection;
        if ($span->end > strlen($this->contents)) {
            throw new InvalidArgumentException('The requested span lies outside the source file.');
        }

        return substr($this->contents, $span->start, $span->length());
    }

    public function getResolvedName(Node|Span $selection): ?ResolvedName
    {
        $span = $selection instanceof Node ? $selection->span : $selection;

        return $this->resolvedNames->find($span->start);
    }

    /**
     * @return list<ResolvedName>
     */
    public function getResolvedNames(Node|Span|null $within = null): array
    {
        $span = $within instanceof Node ? $within->span : $within;

        return $this->resolvedNames->all($span);
    }

    /**
     * @return list<Trivia>
     */
    public function getTrivia(): array
    {
        return $this->trivia->all();
    }
}
