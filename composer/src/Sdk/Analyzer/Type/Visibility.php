<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

/**
 * @api
 */
enum Visibility
{
    case Public;
    case Protected;
    case Private;
}
