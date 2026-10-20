<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Unit\Analyzer;

use Mago\Sdk\Analyzer\EffectiveCallableSignature;
use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\Analyzer\Type\CallableParameter;
use Mago\Sdk\Exception\InvalidArgumentException;
use Mago\Sdk\Internal\Analyzer\Protocol;
use Mago\Sdk\Internal\Protocol\PayloadReader;
use PHPUnit\Framework\TestCase;

final class EffectiveCallableSignatureTest extends TestCase
{
    public function testValidSignatureRetainsItsContract(): void
    {
        $closureThisType = Type::namedObject('ClosureCommand');
        $outType = Type::int();
        $parameter = new CallableParameter(
            '$value',
            Type::string(),
            $closureThisType,
            byReference: true,
            hasDefault: true,
            outType: $outType,
        );
        $signature = new EffectiveCallableSignature([$parameter], false, 'Subject::knownMethod');

        self::assertSame([$parameter], $signature->parameters);
        self::assertSame($closureThisType, $parameter->closureThisType);
        self::assertSame($outType, $parameter->outType);
        self::assertFalse($signature->allowsNamedArguments);
        self::assertSame('Subject::knownMethod', $signature->displayName);
    }

    public function testResponseKeepsOutTypeAndParameterFlags(): void
    {
        $signature = new EffectiveCallableSignature([
            new CallableParameter('$value', Type::string(), byReference: true, hasDefault: true, outType: Type::int()),
        ]);
        $reader = new PayloadReader(Protocol::writeCallableSignatureResponse($signature));
        self::assertSame(0x4D41_4E41, $reader->readU32());
        self::assertSame(0x0002_0002, $reader->readU32());
        self::assertSame(0x800C_0000, $reader->readU32());
        self::assertTrue($reader->readBoolean());
        self::assertFalse($reader->readBoolean());
        self::assertTrue($reader->readBoolean());
        self::assertSame(1, $reader->readU32());
        self::assertTrue($reader->readBoolean());
        self::assertSame('$value', $reader->readBytes());
        self::assertTrue($reader->readBoolean());
        self::assertSame(Type::string()->encode(), $reader->readRaw(1));
        self::assertFalse($reader->readBoolean());
        self::assertTrue($reader->readBoolean());
        self::assertSame(Type::int()->encode(), $reader->readRaw(1));
        self::assertSame(5, $reader->readU8());
        $reader->finish();
    }

    public function testEmptyDisplayNameIsRejected(): void
    {
        $this->expectException(InvalidArgumentException::class);
        $this->expectExceptionMessage('display name cannot be empty');

        new EffectiveCallableSignature([], displayName: '');
    }

    public function testCallableParameterRejectsInvalidVariableName(): void
    {
        $this->expectException(InvalidArgumentException::class);
        $this->expectExceptionMessage('must be a valid PHP variable name');

        new CallableParameter('$');
    }

    public function testDuplicateParameterNameIsRejected(): void
    {
        $this->expectException(InvalidArgumentException::class);
        $this->expectExceptionMessage('is duplicated');

        new EffectiveCallableSignature([new CallableParameter('$value'), new CallableParameter('$value')]);
    }

    public function testVariadicParameterMustBeLast(): void
    {
        $this->expectException(InvalidArgumentException::class);
        $this->expectExceptionMessage('must be last');

        new EffectiveCallableSignature([
            new CallableParameter('$values', variadic: true),
            new CallableParameter('$next'),
        ]);
    }

    public function testRequiredParameterCannotFollowOptionalParameter(): void
    {
        $this->expectException(InvalidArgumentException::class);
        $this->expectExceptionMessage('cannot follow an optional one');

        new EffectiveCallableSignature([
            new CallableParameter('$optional', hasDefault: true),
            new CallableParameter('$required'),
        ]);
    }

    public function testVariadicParameterCannotHaveDefault(): void
    {
        $this->expectException(InvalidArgumentException::class);
        $this->expectExceptionMessage('cannot have a default');

        new EffectiveCallableSignature([new CallableParameter('$values', variadic: true, hasDefault: true)]);
    }
}
