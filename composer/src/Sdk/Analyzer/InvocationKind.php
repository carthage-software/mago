<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * The source-level form of an analyzer provider invocation.
 *
 * @api
 */
enum InvocationKind
{
    case Function;
    case InstanceMethod;
    case StaticMethod;
}
