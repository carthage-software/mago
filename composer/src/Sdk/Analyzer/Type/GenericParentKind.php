<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

/**
 * @api
 */
enum GenericParentKind
{
    case ClassLike;
    case FunctionLike;
}
