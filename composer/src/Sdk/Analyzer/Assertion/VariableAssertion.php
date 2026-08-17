<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Assertion;

use Mago\Sdk\Exception\InvalidArgumentException;

/**
 * An assertion carrying the tracked variable on the other side of a comparison.
 *
 * @api
 */
final class VariableAssertion implements Assertion
{
    public function __construct(
        public readonly VariableAssertionKind $kind,
        public readonly string $variable,
    ) {
        if ($variable === '') {
            throw new InvalidArgumentException('An assertion variable cannot be empty.');
        }
    }
}
