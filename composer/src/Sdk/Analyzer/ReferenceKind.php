<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * The semantic role of a symbol reference.
 *
 * @api
 */
enum ReferenceKind: int
{
    case Body = 1;
    case Signature = 2;
    case OverriddenMember = 3;
    case FunctionLikeReturn = 4;
    case PropertyRead = 5;
    case PropertyWrite = 6;
}
