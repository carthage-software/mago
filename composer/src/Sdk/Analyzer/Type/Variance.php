<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

/** @api */
enum Variance
{
    case Invariant;
    case Covariant;
    case Contravariant;
    case Bivariant;
}
