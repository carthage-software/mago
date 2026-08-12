<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Definition;

use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\Internal\Analyzer\DefinitionName;

/**
 * A parameter declared by an extension-provided function or method.
 *
 * @api
 * @mago-expect lint:excessive-parameter-list
 */
final class ParameterDefinition
{
    public function __construct(
        public readonly string $name,
        public readonly ?Type $type = null,
        public readonly ?Type $nativeType = null,
        public readonly ?Type $outType = null,
        public readonly ?Type $closureThisType = null,
        public readonly ?Type $defaultType = null,
        public readonly int $flags = 0,
    ) {
        DefinitionName::assertVariable($name, 'A parameter definition name');
    }
}
