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
    /**
     * Native analyzer issue codes this hook may filter.
     *
     * Mago uses these targets to avoid sending unrelated diagnostics across
     * the extension boundary.
     *
     * @return non-empty-list<non-empty-string>
     */
    public function getCodes(): array;

    public function filterIssue(IssueFilterContext $context): IssueFilterDecision;
}
