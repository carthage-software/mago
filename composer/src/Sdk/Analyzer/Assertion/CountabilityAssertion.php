<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Assertion;

/**
 * A countability assertion retaining whether Mago can safely negate it.
 *
 * @api
 */
final class CountabilityAssertion implements Assertion
{
    public function __construct(
        public readonly CountabilityAssertionKind $kind,
        public readonly bool $negatable,
    ) {}
}
