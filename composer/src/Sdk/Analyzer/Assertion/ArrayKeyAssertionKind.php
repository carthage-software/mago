<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Assertion;

/**
 * The relationship between an asserted array and a particular key.
 *
 * @api
 */
enum ArrayKeyAssertionKind
{
    case HasKey;
    case DoesNotHaveKey;
    case HasNonnullEntryForKey;
    case DoesNotHaveNonnullEntryForKey;
}
