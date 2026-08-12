<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Definition;

use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\Internal\Analyzer\DefinitionName;

/**
 * A case declared by an extension-provided enum.
 *
 * @api
 */
final class EnumCaseDefinition
{
    public function __construct(
        public readonly string $name,
        public readonly ?Type $valueType = null,
        public readonly int $flags = 0,
    ) {
        DefinitionName::assertClassConstant($name, 'An enum case definition name');
    }
}
