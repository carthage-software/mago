<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Unit\Analyzer;

use Mago\Sdk\Exception\ProtocolException;
use Mago\Sdk\Internal\Analyzer\Protocol;
use Mago\Sdk\Internal\Protocol\PayloadWriter;
use PHPUnit\Framework\TestCase;

use function pack;

final class ThrowTypeProviderTest extends TestCase
{
    public function testEmptyExceptionSetIsHandledWhileNullFallsThrough(): void
    {
        $unhandled = Protocol::writeThrowTypeResponse(null);
        [$unhandledKind, $unhandledReader] = Protocol::readRequest($unhandled);
        self::assertSame(0x8014, $unhandledKind);
        self::assertFalse($unhandledReader->readBoolean());
        $unhandledReader->finish();

        $handledEmpty = Protocol::writeThrowTypeResponse([]);
        [$handledKind, $handledReader] = Protocol::readRequest($handledEmpty);
        self::assertSame(0x8014, $handledKind);
        self::assertTrue($handledReader->readBoolean());
        self::assertSame(0, $handledReader->readCount(1));
        $handledReader->finish();
    }

    public function testThrowTypeResponsePreservesExceptionNames(): void
    {
        $payload = Protocol::writeThrowTypeResponse(['\\DomainException', 'RuntimeException']);
        [$kind, $reader] = Protocol::readRequest($payload);

        self::assertSame(0x8014, $kind);
        self::assertTrue($reader->readBoolean());
        self::assertSame(2, $reader->readCount(2));
        self::assertSame('\\DomainException', $reader->readBytes());
        self::assertSame('RuntimeException', $reader->readBytes());
        $reader->finish();
    }

    public function testThrowTypeResponseRejectsInvalidExceptionNames(): void
    {
        $this->expectException(ProtocolException::class);

        // This deliberately violates the static contract to exercise the runtime guard.
        // @mago-ignore analysis:possibly-invalid-argument
        Protocol::writeThrowTypeResponse(['']);
    }

    public function testFunctionThrowRequestCarriesInvocationAndArguments(): void
    {
        $writer = new PayloadWriter();
        $writer->writeU64(42);
        $writer->writeU8(1);
        $writer->writeU16(1);
        $writer->writeU16(3);
        $writer->writeBytes('Acme\\run');
        $writer->writeU32(2);
        $writer->writeU32(19);
        $writer->writeU16(1);
        $writer->writeBoolean(false);
        $writer->writeBoolean(false);
        $writer->writeBoolean(false);
        $writer->writeU32(7);
        $writer->writeU32(13);
        $writer->writeBytes('$input');
        $writer->writeBoolean(false);
        $payload = pack('N3', 0x4D41_4E41, 0x0002_0002, 20 << 16) . $writer->finish();

        [, $reader] = Protocol::readRequest($payload);
        $request = Protocol::readReturnTypeRequest($reader);

        self::assertSame(42, $request->generation);
        self::assertSame([3], $request->providerIndices);
        self::assertSame('Acme\\run', $request->invocation->name);
        self::assertSame(2, $request->invocation->span->start);
        self::assertSame(19, $request->invocation->span->end);
        self::assertCount(1, $request->invocation->arguments);
        self::assertSame('$input', $request->invocation->arguments[0]->expression);
    }

    public function testExpressionRequestRetainsSourceSpanAndInferredTypeAbsence(): void
    {
        $writer = new PayloadWriter();
        $writer->writeU64(42);
        $writer->writeU16(1);
        $writer->writeU16(0);
        $writer->writeBytes('src/example.php');
        $writer->writeU32(10);
        $writer->writeU32(22);
        $writer->writeBytes('$value[0]');
        $writer->writeBoolean(false);
        $payload = pack('N3', 0x4D41_4E41, 0x0002_0002, 21 << 16) . $writer->finish();

        [, $reader] = Protocol::readRequest($payload);
        $request = Protocol::readExpressionThrowTypeRequest($reader);

        self::assertSame(42, $request->generation);
        self::assertSame([0], $request->providerIndices);
        self::assertSame('src/example.php', $request->sourceFilePath);
        self::assertSame(10, $request->span->start);
        self::assertSame(22, $request->span->end);
        self::assertSame('$value[0]', $request->expression);
        self::assertNull($request->expressionType);
    }
}
