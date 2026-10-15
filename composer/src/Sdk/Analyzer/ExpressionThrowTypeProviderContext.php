<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\CancellationTokenInterface;
use Mago\Sdk\PHPVersion;
use Mago\Sdk\Span;

/**
 * Context passed to providers that describe exceptions thrown by an expression.
 *
 * @api
 */
final class ExpressionThrowTypeProviderContext
{
    /** @mago-expect lint:excessive-parameter-list */
    public function __construct(
        public readonly PHPVersion $phpVersion,
        public readonly Codebase $codebase,
        public readonly TypeComparator $types,
        public readonly CancellationTokenInterface $cancellation,
        public readonly string $sourceFilePath,
        public readonly Span $span,
        public readonly string $expression,
        public readonly ?Type $expressionType,
    ) {}
}
