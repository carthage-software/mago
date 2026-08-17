<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\Exception\InvalidArgumentException;
use Mago\Sdk\Span;

use function in_array;

/**
 * @api
 * @mago-expect lint:cyclomatic-complexity
 * @mago-expect lint:excessive-parameter-list
 */
final class Invocation
{
    /** @var non-empty-string */
    public readonly string $name;

    /** @var null|non-empty-string */
    public readonly ?string $declaringClass;

    /**
     * @param list<Argument> $arguments
     */
    public function __construct(
        public readonly InvocationKind $kind,
        string $name,
        ?string $declaringClass,
        public readonly ?Type $receiverType,
        public readonly Span $span,
        public readonly array $arguments,
    ) {
        if ($name === '') {
            throw new InvalidArgumentException('An analyzer invocation name cannot be empty.');
        }

        if ($declaringClass === '') {
            throw new InvalidArgumentException('An analyzer declaring class cannot be empty.');
        }

        if ($kind === InvocationKind::Function && ($declaringClass !== null || $receiverType !== null)) {
            throw new InvalidArgumentException('A function invocation cannot have a declaring class or receiver type.');
        }

        if ($kind !== InvocationKind::Function && ($declaringClass === null || $receiverType === null)) {
            throw new InvalidArgumentException('A method invocation requires a declaring class and receiver type.');
        }

        $this->name = $name;
        $this->declaringClass = $declaringClass;
    }

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
