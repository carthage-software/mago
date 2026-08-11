<?php

declare(strict_types=1);

namespace Mago\Sdk\Linter;

use Mago\Sdk\CancellationTokenInterface;
use Mago\Sdk\Reporting\Issue;
use Mago\Sdk\Syntax\Node;
use Mago\Sdk\Syntax\ResolvedName;
use Mago\Sdk\Syntax\SourceFile;

/**
 * Context supplied to one custom-rule invocation.
 *
 * @api
 */
final class LintContext
{
    /**
     * @var list<Issue>
     * @internal
     */
    public array $issues = [];

    public function __construct(
        public readonly SourceFile $file,
        public readonly Node $node,
        public readonly CancellationTokenInterface $cancellation,
    ) {}

    public function report(Issue $issue): void
    {
        $this->issues[] = $issue;
    }

    public function getParent(): ?Node
    {
        return $this->file->getParent($this->node);
    }

    /**
     * @return list<Node>
     */
    public function getChildren(): array
    {
        return $this->file->getChildren($this->node);
    }

    public function getText(): string
    {
        return $this->file->getText($this->node);
    }

    public function getResolvedName(): ?ResolvedName
    {
        return $this->file->getResolvedName($this->node);
    }
}
