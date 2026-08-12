<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Definition;

use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\Analyzer\Type\Visibility;
use Mago\Sdk\Internal\Analyzer\DefinitionName;

/**
 * A method declared by an extension-provided class-like symbol.
 *
 * @api
 * @mago-expect lint:excessive-parameter-list
 */
final class MethodDefinition
{
    /**
     * @param list<ParameterDefinition> $parameters
     * @param list<Type> $throws
     * @param list<TemplateDefinition> $templates
     */
    public function __construct(
        public readonly string $name,
        public readonly array $parameters = [],
        public readonly ?Type $returnType = null,
        public readonly ?Type $nativeReturnType = null,
        public readonly array $throws = [],
        public readonly Visibility $visibility = Visibility::Public,
        public readonly array $templates = [],
        public readonly int $flags = 0,
    ) {
        DefinitionName::assertIdentifier($name, 'A method definition name');
    }
}
