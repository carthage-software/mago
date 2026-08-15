<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\Exception\InvalidArgumentException;
use Mago\Sdk\Span;

/**
 * One instance-property access presented to a property type provider.
 *
 * @api
 */
final class PropertyAccess
{
    /** @var non-empty-string */
    public readonly string $class;

    /** @var non-empty-string */
    public readonly string $property;

    public function __construct(
        string $class,
        string $property,
        public readonly PropertyAccessKind $kind,
        public readonly Type $receiverType,
        public readonly Span $span,
    ) {
        if ($class === '' || $property === '') {
            throw new InvalidArgumentException('Analyzer property access class and property cannot be empty.');
        }

        if ($property[0] === '$') {
            throw new InvalidArgumentException('Analyzer property access names must not begin with `$`.');
        }

        $this->class = $class;
        $this->property = $property;
    }
}
