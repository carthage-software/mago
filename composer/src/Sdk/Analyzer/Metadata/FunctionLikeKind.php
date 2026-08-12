<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Metadata;

/** @api */
enum FunctionLikeKind
{
    case Function_;
    case Method;
    case Closure;
    case ArrowFunction;
}
