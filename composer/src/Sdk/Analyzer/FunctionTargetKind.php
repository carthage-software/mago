<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/** @api */
enum FunctionTargetKind: int
{
    case Exact = 1;
    case Prefix = 2;
    case Namespace = 3;
}
