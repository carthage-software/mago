<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Unit\Linter;

use Mago\Sdk\Exception\InvalidArgumentException;
use Mago\Sdk\Internal\Linter\RuleOptions;
use Mago\Sdk\Internal\SignalCancellationToken;
use Mago\Sdk\Linter\LintContext;
use Mago\Sdk\Linter\RuleDefinition;
use Mago\Sdk\Linter\RuleOption;
use Mago\Sdk\Reporting\Level;
use Mago\Sdk\Syntax\NodeKind;
use Mago\Sdk\Testing\SourceFileBuilder;
use PHPUnit\Framework\TestCase;

final class RuleOptionsTest extends TestCase
{
    public function testDuplicateOptionsAreRejected(): void
    {
        $this->expectException(InvalidArgumentException::class);
        $this->expectExceptionMessage('declares option `limit` more than once');

        $this->definition([new RuleOption('limit', 'Maximum count.', 5), new RuleOption('limit', 'Other.', 6)]);
    }

    public function testEmptyOptionNameIsRejected(): void
    {
        $this->expectException(InvalidArgumentException::class);

        new RuleOption('', 'Maximum count.', 5);
    }

    public function testResolveMergesOverridesOverDefaults(): void
    {
        $definition = $this->definition([new RuleOption('limit', 'Maximum count.', 5)]);

        self::assertSame(['limit' => 5], RuleOptions::resolve($definition));
        self::assertSame(['limit' => 7], RuleOptions::resolve($definition, ['limit' => 7]));
    }

    public function testResolveRejectsUnknownOptions(): void
    {
        $this->expectException(InvalidArgumentException::class);
        $this->expectExceptionMessage('has no option named `other`');

        RuleOptions::resolve($this->definition([new RuleOption('limit', 'Maximum count.', 5)]), ['other' => 1]);
    }

    public function testResolveRejectsMismatchedTypes(): void
    {
        $this->expectException(InvalidArgumentException::class);
        $this->expectExceptionMessage('expects int, got string');

        RuleOptions::resolve($this->definition([new RuleOption('limit', 'Maximum count.', 5)]), ['limit' => 'many']);
    }

    public function testGetOptionReturnsResolvedValues(): void
    {
        $builder = new SourceFileBuilder(contents: '<?php foo();');
        $node = $builder->addNode(NodeKind::FunctionCall, 6, 11);
        $builder->addTarget($node);
        $file = $builder->build();

        $context = new LintContext($file, $file->getNode($node), new SignalCancellationToken(), ['limit' => 5]);

        self::assertSame(5, $context->getOption('limit'));
    }

    public function testGetOptionRejectsUnknownNames(): void
    {
        $builder = new SourceFileBuilder(contents: '<?php foo();');
        $node = $builder->addNode(NodeKind::FunctionCall, 6, 11);
        $builder->addTarget($node);
        $file = $builder->build();

        $context = new LintContext($file, $file->getNode($node), new SignalCancellationToken());

        $this->expectException(InvalidArgumentException::class);
        $this->expectExceptionMessage('Unknown linter rule option `limit`');

        $context->getOption('limit');
    }

    /**
     * @param list<RuleOption> $options
     */
    private function definition(array $options): RuleDefinition
    {
        return new RuleDefinition(
            code: 'acme/options-test',
            name: 'Options test',
            description: 'Exercises rule options.',
            defaultLevel: Level::Note,
            defaultEnabled: true,
            targets: [NodeKind::FunctionCall],
            options: $options,
        );
    }
}
