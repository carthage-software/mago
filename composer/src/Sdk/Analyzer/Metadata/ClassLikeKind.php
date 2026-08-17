<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Metadata;

/**
 * @api
 */
enum ClassLikeKind
{
    case Class_;
    case Enum;
    case Trait;
    case Interface;
}
