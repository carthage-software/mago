<?php

declare(strict_types=1);

namespace Mago\Sdk\Linter;

use Mago\Sdk\CancellationTokenInterface;
use Mago\Sdk\Exception\InvalidArgumentException;
use Mago\Sdk\Reporting\Issue;
use Mago\Sdk\Syntax\Node;
use Mago\Sdk\Syntax\ResolvedName;
use Mago\Sdk\Syntax\SourceFile;

use function array_key_exists;

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
        /**
         * @var array<string, string|int|float|bool|null>
         */
        public readonly array $options = [],
    ) {}

    public function report(Issue $issue): void
    {
        $this->issues[] = $issue;
    }

    /**
     * Returns the resolved value of one declared rule option.
     *
     * Values are the schema defaults until Mago forwards explicit configuration.
     *
     * @return string|int|float|bool|null
     */
    public function getOption(string $name): string|int|float|bool|null
    {
        if (!array_key_exists($name, $this->options)) {
            throw new InvalidArgumentException("Unknown linter rule option `{$name}`.");
        }

        return $this->options[$name];
    }

    public function getEnclosingFunction(): ?Node
    {
        return $this->file->getEnclosingFunction($this->node);
    }

    public function getEnclosingClassLike(): ?Node
    {
        return $this->file->getEnclosingClassLike($this->node);
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
