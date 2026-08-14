<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

/**
 * @api
 */
enum ReferenceTypeKind
{
    case Symbol;
    case Member;
    case Global;
}
