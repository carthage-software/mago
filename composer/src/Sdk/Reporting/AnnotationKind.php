<?php

declare(strict_types=1);

namespace Mago\Sdk\Reporting;

/**
 * The visual role of an issue annotation.
 *
 * @api
 */
enum AnnotationKind: int
{
    case Primary = 1;
    case Secondary = 2;
}
