<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Assertion;

use Mago\Sdk\Exception\InvalidArgumentException;

/**
 * An assertion carrying an integer bound, string-length bound, or collection count.
 *
 * @api
 */
final class IntegerAssertion implements Assertion
{
    public function __construct(
        public readonly IntegerAssertionKind $kind,
        public readonly int $value,
    ) {
        if ($kind->isCount() && $value < 0) {
            throw new InvalidArgumentException('An assertion count cannot be negative.');
        }
    }
}
