<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

use Mago\Sdk\Exception\InvalidArgumentException;

/**
 * @api
 */
final class CallableType implements AtomicType
{
    public function __construct(
        public readonly ?CallableSignature $signature,
        public readonly ?FunctionLikeIdentifier $alias,
    ) {
        if (($signature === null) === ($alias === null)) {
            throw new InvalidArgumentException('A callable type requires exactly one signature or alias.');
        }
    }

    public function __toString(): string
    {
        return 'callable';
    }
}
