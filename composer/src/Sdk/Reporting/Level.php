<?php

declare(strict_types=1);

namespace Mago\Sdk\Reporting;

/**
 * Severity assigned to a reported issue.
 *
 * @api
 */
enum Level: int
{
    case Note = 1;
    case Help = 2;
    case Warning = 3;
    case Error = 4;
}
