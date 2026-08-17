<?php

declare(strict_types=1);

namespace Mago\Sdk\Syntax;

use Mago\Sdk\Exception\InvalidArgumentException;
use Mago\Sdk\Internal\Syntax\LiteralStringStore;
use Mago\Sdk\Internal\Syntax\NodeStore;
use Mago\Sdk\Internal\Syntax\ResolvedNameStore;
use Mago\Sdk\Internal\Syntax\TriviaStore;
use Mago\Sdk\PHPVersion;
use Mago\Sdk\Span;

use function array_pop;
use function count;
use function strlen;
use function substr;

/**
 * An immutable syntax view of the exact source analyzed by Mago.
 *
 * @api
 * @mago-expect lint:cyclomatic-complexity
 * @mago-expect lint:too-many-methods
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
        private readonly ?LiteralStringStore $literalStrings,
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

    /**
     * @return list<Node>
     */
    public function getAncestors(Node $node): array
    {
        $ancestors = [];
        while (($node = $this->getParent($node)) !== null) {
            $ancestors[] = $node;
        }

        return $ancestors;
    }

    /**
     * Returns descendants in source order, optionally restricted to one kind.
     *
     * @return list<Node>
     */
    public function getDescendants(Node $node, ?NodeKind $kind = null): array
    {
        $descendants = [];
        $stack = [$node];
        while (($current = array_pop($stack)) !== null) {
            $children = $this->getChildren($current);
            for ($index = count($children) - 1; $index >= 0; --$index) {
                $stack[] = $children[$index];
            }
            if ($current !== $node && ($kind === null || $current->kind === $kind)) {
                $descendants[] = $current;
            }
        }

        return $descendants;
    }

    /**
     * Returns the first depth-first descendant of the requested kind.
     */
    public function getFirstDescendant(Node $node, NodeKind $kind): ?Node
    {
        $stack = [];
        $children = $this->getChildren($node);
        for ($index = count($children) - 1; $index >= 0; --$index) {
            $stack[] = $children[$index];
        }

        while (($current = array_pop($stack)) !== null) {
            if ($current->kind === $kind) {
                return $current;
            }

            $children = $this->getChildren($current);
            for ($index = count($children) - 1; $index >= 0; --$index) {
                $stack[] = $children[$index];
            }
        }

        return null;
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
     * Returns the decoded value of a literal-string node.
     *
     * Snapshots that do not request decoded literals return `null`.
     */
    public function getLiteralString(Node $node): ?string
    {
        return $this->literalStrings?->find($node->id);
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
