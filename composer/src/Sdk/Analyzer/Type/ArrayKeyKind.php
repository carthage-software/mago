<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

/**
 * @api
 */
enum ArrayKeyKind
{
    case Integer;
    case String;
    case ClassLikeConstant;
}
