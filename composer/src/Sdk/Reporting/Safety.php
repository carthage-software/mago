<?php

declare(strict_types=1);

namespace Mago\Sdk\Reporting;

/**
 * How safe it is to apply a suggested text edit automatically.
 *
 * @api
 */
enum Safety: int
{
    case Safe = 1;
    case PotentiallyUnsafe = 2;
    case Unsafe = 3;
}
