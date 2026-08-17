<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

/**
 * @api
 */
enum IntegerTypeKind
{
    case Literal;
    case From;
    case To;
    case Range;
    case General;
    case UnspecifiedLiteral;
}
