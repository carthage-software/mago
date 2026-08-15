<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\Internal\Analyzer\DefinitionName;

/**
 * @api
 * @mago-expect lint:excessive-parameter-list
 */
final class CallableParameter
{
    public function __construct(
        public readonly ?string $name = null,
        public readonly ?Type $type = null,
        public readonly ?Type $closureThisType = null,
        public readonly bool $byReference = false,
        public readonly bool $variadic = false,
        public readonly bool $hasDefault = false,
    ) {
        if ($name !== null) {
            DefinitionName::assertVariable($name, 'A callable parameter name');
        }
    }
}
