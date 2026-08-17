<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\CancellationTokenInterface;
use Mago\Sdk\PHPVersion;

/**
 * Context available while selecting a callable's effective signature.
 *
 * Argument expressions have not been analyzed yet, so their `type` fields are
 * null. Their source text, names, flags, and spans remain available.
 *
 * @api
 */
final class CallableSignatureProviderContext
{
    public function __construct(
        public readonly PHPVersion $phpVersion,
        public readonly Codebase $codebase,
        public readonly Invocation $invocation,
        public readonly TypeComparator $types,
        public readonly CancellationTokenInterface $cancellation,
    ) {}
}
