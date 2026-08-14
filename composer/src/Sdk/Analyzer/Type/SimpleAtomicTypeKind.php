<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

/**
 * @api
 */
enum SimpleAtomicTypeKind: string
{
    case Never = 'never';
    case Null = 'null';
    case Void = 'void';
    case Placeholder = '_';
}
