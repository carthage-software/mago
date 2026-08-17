<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * A native relationship Mago should evaluate between two semantic types.
 *
 * @api
 */
enum TypeComparisonKind: int
{
    case Equal = 1;
    case ContainedBy = 2;
    case CanBeIdentical = 3;
}
