<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

/** @api */
enum ClassLikeStringVariant
{
    case Any;
    case Generic;
    case Literal;
    case OfType;
}
