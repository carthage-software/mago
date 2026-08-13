<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Fixture;

use Mago\Sdk\Linter\LintContext;
use Mago\Sdk\Linter\Rule;
use Mago\Sdk\Linter\RuleDefinition;
use Mago\Sdk\Reporting\Issue;
use Mago\Sdk\Reporting\Level;
use Mago\Sdk\Reporting\TextEdit;
use Mago\Sdk\Syntax\NodeKind;

use function strcasecmp;

final class PreferArrayAnyRule implements Rule
{
    public function getDefinition(): RuleDefinition
    {
        return new RuleDefinition(
            code: 'mago-sdk-test/prefer-array-any',
            name: 'Prefer array_any',
            description: 'Suggests array_any instead of Psl\\Iter\\any.',
            defaultLevel: Level::Help,
            defaultEnabled: true,
            targets: [NodeKind::FunctionCall],
        );
    }

    public function lint(LintContext $context): void
    {
        $resolvedName = $context->getResolvedName();
        if ($resolvedName === null || strcasecmp($resolvedName->name, 'Psl\\Iter\\any') !== 0) {
            return;
        }

        $context->report(Issue::new('Prefer array_any() over Psl\\Iter\\any().', $context->node->span)->withHelp(
            'Replace this call with array_any().',
        )->withEdit(TextEdit::replace($resolvedName->span, 'array_any')));
    }
}
