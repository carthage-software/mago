<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

/**
 * @api
 */
enum MixedTruthiness
{
    case Undetermined;
    case Truthy;
    case Falsy;
}
