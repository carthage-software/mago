<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * Whether a local variable exists immediately before a targeted node executes.
 *
 * @api
 */
enum VariableDefinedness
{
    case Undefined;
    case PossiblyDefined;
    case Defined;
}
