<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Fixture;

use Mago\Sdk\Linter\LintContext;
use Mago\Sdk\Linter\Rule;
use Mago\Sdk\Linter\RuleDefinition;
use Mago\Sdk\Reporting\Issue;
use Mago\Sdk\Reporting\Level;
use Mago\Sdk\Syntax\NodeKind;

final class NoInterfaceRule implements Rule
{
    public function getDefinition(): RuleDefinition
    {
        return new RuleDefinition(
            code: 'mago-sdk-test/no-interface',
            name: 'No interfaces',
            description: 'Rejects interface declarations for the SDK integration test.',
            defaultLevel: Level::Warning,
            defaultEnabled: true,
            targets: [NodeKind::Interface],
        );
    }

    public function lint(LintContext $context): void
    {
        $context->report(Issue::new('Interfaces are forbidden by this integration-test rule.', $context->node->span));
    }
}
