<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Metadata;

/** @api */
enum CallableInvocationKind
{
    case Immediately;
    case Later;
}
