<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

/** @api */
final class CallableType implements AtomicType
{
    public function __construct(
        public readonly ?CallableSignature $signature,
        public readonly ?FunctionLikeIdentifier $alias,
    ) {}

    public function __toString(): string
    {
        return 'callable';
    }
}
