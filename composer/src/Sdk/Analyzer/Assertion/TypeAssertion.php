<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Assertion;

use Mago\Sdk\Analyzer\Type;

/**
 * An assertion carrying the type or value set involved in the relationship.
 *
 * @api
 */
final class TypeAssertion implements Assertion
{
    public function __construct(
        public readonly TypeAssertionKind $kind,
        public readonly Type $type,
    ) {}
}
