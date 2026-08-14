<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

/**
 * @api
 */
enum FunctionLikeKind
{
    case Function_;
    case Method;
    case Closure;
}
