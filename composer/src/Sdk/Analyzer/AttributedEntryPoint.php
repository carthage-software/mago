<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\Exception\InvalidArgumentException;

/**
 * Selects attributed methods that a framework invokes externally.
 *
 * Exact classes also match their subclasses and implementations.
 *
 * @api
 */
final class AttributedEntryPoint
{
    public readonly ClassTarget $class;

    /**
     * @var non-empty-string
     */
    public readonly string $attribute;

    public function __construct(string|ClassTarget $class, string $attribute)
    {
        if ($attribute === '') {
            throw new InvalidArgumentException('An attributed entry point requires an attribute class name.');
        }

        $this->class = $class instanceof ClassTarget ? $class : ClassTarget::exact($class);
        $this->attribute = $attribute;
    }
}
