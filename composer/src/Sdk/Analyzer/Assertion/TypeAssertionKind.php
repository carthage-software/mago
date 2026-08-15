<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Assertion;

/**
 * The relationship between an asserted expression and a semantic type.
 *
 * @api
 */
enum TypeAssertionKind
{
    case IsType;
    case IsNotType;
    case IsIdentical;
    case IsNotIdentical;
    case IsEqual;
    case IsNotEqual;
    case InArray;
    case NotInArray;
}
