<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Unit\Analyzer;

use Mago\Sdk\Analyzer\InvocationKind;
use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\Exception\ProtocolException;
use Mago\Sdk\Internal\Analyzer\Protocol;
use Mago\Sdk\Internal\Analyzer\ReturnTypeRequest;
use Mago\Sdk\Internal\Analyzer\TypeCodec;
use Mago\Sdk\Internal\Protocol\PayloadWriter;
use PHPUnit\Framework\Attributes\DataProvider;
use PHPUnit\Framework\TestCase;
use Throwable;

use function pack;
use function substr;

/**
 * @mago-expect lint:too-many-methods
 */
final class InvocationTest extends TestCase
{
    /**
     * @return iterable<string, array{int, InvocationKind, null|string, null|string}>
     */
    public static function validInvocations(): iterable
    {
        yield 'function' => [1, InvocationKind::Function, null, null];
        yield 'instance method' => [2, InvocationKind::InstanceMethod, 'BaseModel', 'User'];
        yield 'static method' => [3, InvocationKind::StaticMethod, 'BaseModel', 'User'];
    }

    #[DataProvider('validInvocations')]
    public function testInvocationContextRoundTrips(
        int $encodedKind,
        InvocationKind $kind,
        ?string $declaringClass,
        ?string $receiver,
    ): void {
        $request = self::decode(self::request(
            $encodedKind,
            $declaringClass,
            $receiver === null ? null : Type::namedObject($receiver),
        ));

        self::assertSame($kind, $request->invocation->kind);
        self::assertSame('target', $request->invocation->name);
        self::assertSame($declaringClass, $request->invocation->declaringClass);
        self::assertSame(
            $receiver,
            $request->invocation->receiverType === null ? null : (string) $request->invocation->receiverType,
        );
    }

    public function testUnknownInvocationKindIsRejected(): void
    {
        $this->expectException(ProtocolException::class);
        $this->expectExceptionMessage('Unknown analyzer invocation kind 255.');

        self::decode(self::request(255));
    }

    public function testReceiverRetainsItsSnapshotHandle(): void
    {
        $request = self::decode(self::request(2, 'BaseModel', Type::namedObject('User')));
        $receiver = $request->invocation->receiverType;
        self::assertNotNull($receiver);

        $response = Protocol::writeReturnTypeResponse($receiver);

        self::assertSame(pack('CN', 0, 0), substr($response, 13));
    }

    public function testMethodWithoutReceiverIsRejected(): void
    {
        $writer = self::requestPrefix(2);
        $writer->writeBytes('BaseModel');
        $writer->writeBytes('target');
        $writer->writeU32(1);
        $writer->writeU32(2);
        $writer->writeU16(0);

        $this->expectException(Throwable::class);
        self::decode(self::message($writer));
    }

    public function testFunctionWithReceiverIsRejected(): void
    {
        $writer = self::requestPrefix(1);
        $writer->writeBytes('target');
        self::writeReceiver($writer, Type::namedObject('Unexpected'));
        $writer->writeU32(1);
        $writer->writeU32(2);
        $writer->writeU16(0);

        $this->expectException(Throwable::class);
        self::decode(self::message($writer));
    }

    public function testTruncatedReceiverIsRejected(): void
    {
        $writer = self::requestPrefix(2);
        $writer->writeBytes('BaseModel');
        $writer->writeBytes('target');
        self::writeReceiver($writer, Type::namedObject('User'));

        $this->expectException(Throwable::class);
        self::decode(self::messagePayload(substr($writer->finish(), 0, -3) . "\x02"));
    }

    public function testEmptyMethodDeclaringClassIsRejected(): void
    {
        $this->expectException(ProtocolException::class);
        $this->expectExceptionMessage('Analyzer invocation names cannot be empty.');

        self::decode(self::request(2, '', Type::namedObject('User')));
    }

    private static function request(int $kind, ?string $declaringClass = null, ?Type $receiver = null): string
    {
        $writer = self::requestPrefix($kind);
        if ($declaringClass !== null) {
            $writer->writeBytes($declaringClass);
        }
        $writer->writeBytes('target');
        if ($receiver !== null) {
            self::writeReceiver($writer, $receiver);
        }
        $writer->writeU32(1);
        $writer->writeU32(2);
        $writer->writeU16(0);

        return self::message($writer);
    }

    private static function requestPrefix(int $kind): PayloadWriter
    {
        $writer = new PayloadWriter();
        $writer->writeU64(1);
        $writer->writeU8($kind);
        $writer->writeU16(1);
        $writer->writeU16(0);

        return $writer;
    }

    private static function writeReceiver(PayloadWriter $writer, Type $receiver): void
    {
        $writer->writeU32(0);
        TypeCodec::writeComplete($writer, $receiver);
    }

    private static function message(PayloadWriter $writer): string
    {
        return self::messagePayload($writer->finish());
    }

    private static function messagePayload(string $payload): string
    {
        return pack('N3', 0x4D41_4E41, 0x0001_0000, 2 << 16) . $payload;
    }

    private static function decode(string $payload): ReturnTypeRequest
    {
        [, $reader] = Protocol::readRequest($payload);

        return Protocol::readReturnTypeRequest($reader);
    }
}
