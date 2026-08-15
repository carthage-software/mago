<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Assertion;

/**
 * The kind of an assertion that carries no additional value.
 *
 * @api
 */
enum SimpleAssertionKind
{
    case Any;
    case Falsy;
    case Truthy;
    case IsEqualIsset;
    case IsIsset;
    case IsNotIsset;
    case HasStringArrayAccess;
    case HasIntOrStringArrayAccess;
    case ArrayKeyExists;
    case ArrayKeyDoesNotExist;
    case Empty;
    case NonEmpty;
    case EmptyCountable;
    case Countable;
}
