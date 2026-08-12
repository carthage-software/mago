<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Definition;

use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\Analyzer\Type\Visibility;
use Mago\Sdk\Internal\Analyzer\DefinitionName;

/**
 * A property declared by an extension-provided class-like symbol.
 *
 * @api
 * @mago-expect lint:excessive-parameter-list
 */
final class PropertyDefinition
{
    public function __construct(
        public readonly string $name,
        public readonly ?Type $type = null,
        public readonly ?Type $nativeType = null,
        public readonly ?Type $writeType = null,
        public readonly ?Type $defaultType = null,
        public readonly Visibility $readVisibility = Visibility::Public,
        public readonly Visibility $writeVisibility = Visibility::Public,
        public readonly int $flags = 0,
    ) {
        DefinitionName::assertVariable($name, 'A property definition name');
    }
}
