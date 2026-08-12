<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Definition;

use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\Analyzer\Type\Variance;
use Mago\Sdk\Internal\Analyzer\DefinitionName;

/**
 * A generic template parameter declared on a class-like, function, or method.
 *
 * @api
 */
final class TemplateDefinition
{
    public function __construct(
        public readonly string $name,
        public readonly Type $constraint,
        public readonly ?Type $default = null,
        public readonly Variance $variance = Variance::Invariant,
        public readonly bool $readonly = false,
    ) {
        DefinitionName::assertIdentifier($name, 'A template definition name');
    }
}
