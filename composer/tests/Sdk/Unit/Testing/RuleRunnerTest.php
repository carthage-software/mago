<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Unit\Testing;

use Mago\Sdk\Exception\InvalidArgumentException;
use Mago\Sdk\Linter\LintContext;
use Mago\Sdk\Linter\Rule;
use Mago\Sdk\Linter\RuleDefinition;
use Mago\Sdk\Linter\RuleOption;
use Mago\Sdk\Reporting\Issue;
use Mago\Sdk\Reporting\Level;
use Mago\Sdk\Syntax\NodeKind;
use Mago\Sdk\Syntax\SourceFile;
use Mago\Sdk\Testing\RuleRunner;
use Mago\Sdk\Testing\SourceFileBuilder;
use PHPUnit\Framework\TestCase;

use function sprintf;

final class RuleRunnerTest extends TestCase
{
    public function testRunnerVisitsSubscribedTargetsOnly(): void
    {
        $builder = new SourceFileBuilder(contents: '<?php function greet(): void { hello(); }');
        $program = $builder->addNode(NodeKind::Program, 0, 41);
        $function = $builder->addNode(NodeKind::Function, 6, 41, $program);
        $call = $builder->addNode(NodeKind::FunctionCall, 31, 38, $function);
        $literal = $builder->addNode(NodeKind::Literal, 31, 38, $function);
        $builder->addTarget($call);
        $builder->addTarget($literal);
        $builder->addResolvedName(31, 38, 'hello', false);
        $file = $builder->build();

        $issues = RuleRunner::run(new EnclosingScopeRule(), $file);

        self::assertCount(1, $issues);
        self::assertSame('hello:Function:none', $issues[0]->message);
        self::assertSame('hello', $file->getResolvedName($file->getNode($call))?->name);
    }

    public function testRunnerAppliesOptionOverrides(): void
    {
        $file = $this->functionCallFile();

        $issues = RuleRunner::run(new EnclosingScopeRule(), $file, ['greeting' => 'hi']);

        self::assertCount(1, $issues);
        self::assertSame('hi:Function:none', $issues[0]->message);
    }

    public function testRunnerRejectsUnknownOptions(): void
    {
        $this->expectException(InvalidArgumentException::class);

        RuleRunner::run(new EnclosingScopeRule(), $this->functionCallFile(), ['other' => 1]);
    }

    public function testEnclosingScopesResolveThroughMethods(): void
    {
        $builder = new SourceFileBuilder(contents: '<?php class Box { public function open(): void { hello(); } }');
        $program = $builder->addNode(NodeKind::Program, 0, 61);
        $class = $builder->addNode(NodeKind::Class_, 6, 61, $program);
        $method = $builder->addNode(NodeKind::Method, 18, 59, $class);
        $call = $builder->addNode(NodeKind::FunctionCall, 49, 56, $method);
        $builder->addTarget($call);
        $file = $builder->build();

        $issues = RuleRunner::run(new EnclosingScopeRule(), $file);

        self::assertCount(1, $issues);
        self::assertSame('hello:Method:Class', $issues[0]->message);
        self::assertSame($class, $file->getEnclosingClassLike($file->getNode($class))?->id);
        self::assertSame($method, $file->getEnclosingFunction($file->getNode($method))?->id);
    }

    private function functionCallFile(): SourceFile
    {
        $builder = new SourceFileBuilder(contents: '<?php function greet(): void { hello(); }');
        $program = $builder->addNode(NodeKind::Program, 0, 41);
        $function = $builder->addNode(NodeKind::Function, 6, 41, $program);
        $call = $builder->addNode(NodeKind::FunctionCall, 31, 38, $function);
        $builder->addTarget($call);

        return $builder->build();
    }
}

final class EnclosingScopeRule implements Rule
{
    public function getDefinition(): RuleDefinition
    {
        return new RuleDefinition(
            code: 'acme/enclosing-scope-test',
            name: 'Enclosing scope test',
            description: 'Reports the enclosing scopes of each call.',
            defaultLevel: Level::Note,
            defaultEnabled: true,
            targets: [NodeKind::FunctionCall],
            options: [new RuleOption('greeting', 'Word to report.', 'hello')],
        );
    }

    public function lint(LintContext $context): void
    {
        $function = $context->getEnclosingFunction();
        $classLike = $context->getEnclosingClassLike();

        $context->report(
            Issue::new(
                sprintf(
                    '%s:%s:%s',
                    (string) $context->getOption('greeting'),
                    $function === null ? 'none' : $function->kind->value,
                    $classLike === null ? 'none' : $classLike->kind->value,
                ),
                $context->node->span,
            ),
        );
    }
}
