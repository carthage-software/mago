<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\Span;

use function in_array;

/**
 * @api
 */
final class Invocation
{
    /**
     * @param non-empty-string $name
     * @param null|non-empty-string $class
     * @param list<Argument> $arguments
     */
    public function __construct(
        public readonly string $name,
        public readonly ?string $class,
        public readonly Span $span,
        public readonly array $arguments,
    ) {}

    public function getArgument(int $index, string ...$names): ?Argument
    {
        $argument = $this->arguments[$index] ?? null;
        if ($argument !== null && $argument->name === null) {
            return $argument;
        }

        foreach ($this->arguments as $candidate) {
            if ($candidate->name !== null && in_array($candidate->name, $names, true)) {
                return $candidate;
            }
        }

        return null;
    }
}
