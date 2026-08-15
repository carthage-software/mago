<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Unit\Analyzer;

use Mago\Sdk\Analyzer\EffectiveCallableSignature;
use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\Analyzer\Type\CallableParameter;
use Mago\Sdk\Exception\InvalidArgumentException;
use PHPUnit\Framework\TestCase;

final class EffectiveCallableSignatureTest extends TestCase
{
    public function testValidSignatureRetainsItsContract(): void
    {
        $parameter = new CallableParameter('$value', Type::string(), byReference: true, hasDefault: true);
        $signature = new EffectiveCallableSignature([$parameter], false);

        self::assertSame([$parameter], $signature->parameters);
        self::assertFalse($signature->allowsNamedArguments);
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
