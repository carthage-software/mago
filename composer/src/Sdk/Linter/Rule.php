<?php

declare(strict_types=1);

namespace Mago\Sdk\Linter;

/**
 * A custom linter rule executed once for each subscribed syntax node.
 *
 * @api
 */
interface Rule
{
    public function getDefinition(): RuleDefinition;

    public function lint(LintContext $context): void;
}
