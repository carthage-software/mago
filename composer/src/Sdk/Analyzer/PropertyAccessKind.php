<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * The operation being performed on an extension-provided property.
 *
 * @api
 */
enum PropertyAccessKind
{
    case Read;
    case Write;
}
