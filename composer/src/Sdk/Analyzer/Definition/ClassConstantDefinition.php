<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Definition;

use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\Analyzer\Type\Visibility;
use Mago\Sdk\Internal\Analyzer\DefinitionName;

/**
 * A constant declared by an extension-provided class-like symbol.
 *
 * @api
 * @mago-expect lint:excessive-parameter-list
 */
final class ClassConstantDefinition
{
    public function __construct(
        public readonly string $name,
        public readonly ?Type $type = null,
        public readonly ?Type $nativeType = null,
        public readonly ?Type $valueType = null,
        public readonly Visibility $visibility = Visibility::Public,
        public readonly int $flags = 0,
    ) {
        DefinitionName::assertClassConstant($name, 'A class constant definition name');
    }
}
