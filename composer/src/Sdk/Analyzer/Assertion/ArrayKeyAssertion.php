<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Assertion;

use Mago\Sdk\Analyzer\Type\ArrayKey;

/**
 * An assertion carrying the array key involved in the relationship.
 *
 * @api
 */
final class ArrayKeyAssertion implements Assertion
{
    public function __construct(
        public readonly ArrayKeyAssertionKind $kind,
        public readonly ArrayKey $key,
    ) {}
}
