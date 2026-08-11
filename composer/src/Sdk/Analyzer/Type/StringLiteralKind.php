<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

/** @api */
enum StringLiteralKind
{
    case General;
    case Unspecified;
    case Value;
}
