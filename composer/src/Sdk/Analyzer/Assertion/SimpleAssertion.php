<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Assertion;

/**
 * An assertion whose kind completely describes the guaranteed fact.
 *
 * @api
 */
final class SimpleAssertion implements Assertion
{
    public function __construct(
        public readonly SimpleAssertionKind $kind,
    ) {}
}
