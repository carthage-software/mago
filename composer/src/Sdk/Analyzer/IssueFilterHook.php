<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * Last-resort filtering for native analyzer issues that cannot be prevented by a semantic provider.
 *
 * Mago batches all issues for a file into one worker request. The worker invokes
 * this hook locally for each issue, so implementing the per-issue API does not
 * introduce one IPC round trip per diagnostic.
 *
 * @api
 */
interface IssueFilterHook
{
    public function filterIssue(IssueFilterContext $context): IssueFilterDecision;
}
