<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Definition;

use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\Internal\Analyzer\DefinitionName;

/**
 * A global constant declaration contributed directly by an extension.
 *
 * @api
 */
final class ConstantDefinition
{
    public function __construct(
        public readonly string $name,
        public readonly ?Type $type = null,
        public readonly ?Type $valueType = null,
        public readonly int $flags = 0,
    ) {
        DefinitionName::assertSymbol($name, 'A constant definition name');
    }
}
