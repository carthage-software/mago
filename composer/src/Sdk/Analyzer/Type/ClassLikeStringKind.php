<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

/**
 * @api
 */
enum ClassLikeStringKind: string
{
    case Class_ = 'class-string';
    case Interface = 'interface-string';
    case Enum = 'enum-string';
    case Trait = 'trait-string';
}
