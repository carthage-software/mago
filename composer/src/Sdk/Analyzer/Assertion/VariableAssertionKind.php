<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Assertion;

/**
 * The ordering relationship between an asserted expression and another variable.
 *
 * @api
 */
enum VariableAssertionKind
{
    case IsLessThan;
    case IsLessThanOrEqual;
    case IsGreaterThan;
    case IsGreaterThanOrEqual;
}
