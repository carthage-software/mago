<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Unit\Analyzer;

use Closure;
use Mago\Sdk\Analyzer\Definition\ClassConstantDefinition;
use Mago\Sdk\Analyzer\Definition\ClassLikeDefinition;
use Mago\Sdk\Analyzer\Definition\ConstantDefinition;
use Mago\Sdk\Analyzer\Definition\EnumCaseDefinition;
use Mago\Sdk\Analyzer\Definition\FunctionDefinition;
use Mago\Sdk\Analyzer\Definition\MethodDefinition;
use Mago\Sdk\Analyzer\Definition\ParameterDefinition;
use Mago\Sdk\Analyzer\Definition\PropertyDefinition;
use Mago\Sdk\Analyzer\Definition\TemplateDefinition;
use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\Exception\InvalidArgumentException;
use PHPUnit\Framework\Attributes\DataProvider;
use PHPUnit\Framework\TestCase;

final class DefinitionTest extends TestCase
{
    public function testLegalDefinitionNamesAreAccepted(): void
    {
        self::assertSame(
            'Vendor\\enum',
            (new ClassLikeDefinition('Vendor\\enum', parentClass: 'Vendor\\ParentType'))->name,
        );
        self::assertSame('Vendor\\from', (new FunctionDefinition('Vendor\\from'))->name);
        self::assertSame('Vendor\\enum', (new ConstantDefinition('Vendor\\enum'))->name);
        self::assertSame('class', (new MethodDefinition('class'))->name);
        self::assertSame('function', (new ClassConstantDefinition('function'))->name);
        self::assertSame('function', (new EnumCaseDefinition('function'))->name);
        self::assertSame('$é', (new ParameterDefinition('$é'))->name);
        self::assertSame('$this', (new PropertyDefinition('$this'))->name);
        self::assertSame('T1', (new TemplateDefinition('T1', Type::mixed()))->name);
    }

    #[DataProvider('provideInvalidDefinitions')]
    public function testInvalidDefinitionNameIsRejected(Closure $factory): void
    {
        $this->expectException(InvalidArgumentException::class);

        $factory();
    }

    /**
     * @return iterable<string, array{Closure(): object}>
     */
    public static function provideInvalidDefinitions(): iterable
    {
        yield 'reserved class-like name' => [static fn(): object => new ClassLikeDefinition('Vendor\\class')];
        yield 'reserved namespace segment' => [static fn(): object => new ClassLikeDefinition('enum\\ValidName')];
        yield 'invalid parent class' => [
            static fn(): object => new ClassLikeDefinition('ValidName', parentClass: '1Invalid'),
        ];
        yield 'invalid function name' => [static fn(): object => new FunctionDefinition('bad-name')];
        yield 'reserved global constant' => [static fn(): object => new ConstantDefinition('readonly')];
        yield 'qualified method' => [static fn(): object => new MethodDefinition('Vendor\\method')];
        yield 'reserved class constant' => [static fn(): object => new ClassConstantDefinition('CLASS')];
        yield 'reserved enum case' => [static fn(): object => new EnumCaseDefinition('class')];
        yield 'bare parameter dollar' => [static fn(): object => new ParameterDefinition('$')];
        yield 'digit-prefixed parameter' => [static fn(): object => new ParameterDefinition('$1invalid')];
        yield 'invalid property bytes' => [static fn(): object => new PropertyDefinition('$bad-name')];
        yield 'invalid template name' => [static fn(): object => new TemplateDefinition('$T', Type::mixed())];
    }
}
