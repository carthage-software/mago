<?php

declare(strict_types=1);

namespace Mago\Sdk\Testing;

use Mago\Sdk\CancellationTokenInterface;
use Mago\Sdk\Internal\Linter\RuleOptions;
use Mago\Sdk\Internal\SignalCancellationToken;
use Mago\Sdk\Linter\LintContext;
use Mago\Sdk\Linter\Rule;
use Mago\Sdk\Reporting\Issue;
use Mago\Sdk\Syntax\SourceFile;

use function in_array;

/**
 * Runs a linter rule against a snapshot without a worker or host.
 *
 * Only the rule's subscribed node kinds are visited, mirroring worker dispatch.
 * Option overrides are validated against the rule definition; without
 * overrides the schema defaults apply, exactly as in production.
 *
 * @api
 */
final class RuleRunner
{
    /**
     * @param array<string, string|int|float|bool|null> $options
     *
     * @return list<Issue>
     */
    public static function run(
        Rule $rule,
        SourceFile $file,
        array $options = [],
        ?CancellationTokenInterface $cancellation = null,
    ): array {
        $definition = $rule->getDefinition();
        $resolved = RuleOptions::resolve($definition, $options);
        $cancellation ??= new SignalCancellationToken();
        $issues = [];
        foreach ($file->getTargetNodes() as $node) {
            if (!in_array($node->kind, $definition->targets, true)) {
                continue;
            }

            $context = new LintContext($file, $node, $cancellation, $resolved);
            $rule->lint($context);
            foreach ($context->issues as $issue) {
                $issues[] = $issue;
            }
        }

        return $issues;
    }
}
