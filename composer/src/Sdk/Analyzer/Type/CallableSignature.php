<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

use Mago\Sdk\Analyzer\Type;

/**
 * @api
 * @mago-expect lint:excessive-parameter-list
 */
final class CallableSignature
{
    /**
     * @param list<CallableParameter> $parameters
     * @param list<CallableConstraint> $constraints
     */
    public function __construct(
        public readonly bool $pure,
        public readonly bool $closure,
        public readonly array $parameters,
        public readonly ?Type $returnType,
        public readonly ?FunctionLikeIdentifier $source,
        public readonly array $constraints,
    ) {}
}
