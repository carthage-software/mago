<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

/**
 * @api
 */
enum ScalarTypeKind: string
{
    case Scalar = 'scalar';
    case Numeric = 'numeric';
    case ArrayKey = 'array-key';
    case Boolean = 'bool';
    case Integer = 'int';
    case Float = 'float';
    case String = 'string';
    case ClassLikeString = 'class-string';
}
