<?php

declare(strict_types=1);

namespace Mago\Sdk\Syntax;

/**
 * Comment forms included in filtered source snapshots.
 *
 * @api
 */
enum TriviaKind: string
{
    case SingleLineComment = 'SingleLineComment';
    case MultiLineComment = 'MultiLineComment';
    case HashComment = 'HashComment';
    case DocBlockComment = 'DocBlockComment';
}
