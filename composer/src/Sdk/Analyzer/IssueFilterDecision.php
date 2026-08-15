<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * The outcome of filtering one analyzer issue.
 *
 * @api
 */
enum IssueFilterDecision
{
    case Keep;
    case Remove;
}
