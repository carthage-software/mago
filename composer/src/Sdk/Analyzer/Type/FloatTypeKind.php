<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

/** @api */
enum FloatTypeKind
{
    case General;
    case UnspecifiedLiteral;
    case Literal;
}
