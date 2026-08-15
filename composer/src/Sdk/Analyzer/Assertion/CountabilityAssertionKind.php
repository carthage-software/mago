<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Assertion;

/**
 * The kind of countability assertion carrying negation behavior.
 *
 * @api
 */
enum CountabilityAssertionKind
{
    case NonEmpty;
    case NotCountable;
}
