<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * Supplies thrown exception class names for an expression.
 *
 * Return `null` to fall through to other providers or native analysis. Return
 * an empty list when the expression does not throw.
 *
 * @api
 */
interface ExpressionThrowTypeProvider
{
    /** @return list<non-empty-string>|null */
    public function getThrownExceptions(ExpressionThrowTypeProviderContext $context): ?array;
}
