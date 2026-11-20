<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Unit\Analyzer;

use Mago\Sdk\Analyzer\Metadata\CallableInvocationKind;
use Mago\Sdk\Analyzer\Metadata\MetadataFlags;
use Mago\Sdk\Analyzer\Metadata\MethodFields;
use Mago\Sdk\Analyzer\Metadata\ParameterMetadata;
use Mago\Sdk\Internal\Analyzer\MetadataCodec;
use Mago\Sdk\Internal\Protocol\PayloadReader;
use Mago\Sdk\Internal\Protocol\PayloadWriter;
use Mago\Sdk\SourceLocation;
use Mago\Sdk\Span;
use PHPUnit\Framework\TestCase;

final class CallableInvocationKindTest extends TestCase
{
    public function testTimingUsesExistingMetadataFlagBits(): void
    {
        $location = new SourceLocation('example.php', new Span(0, 1));
        $cases = [
            [0, null],
            [MetadataFlags::IMMEDIATELY_INVOKED_CALLABLE, CallableInvocationKind::Immediately],
            [MetadataFlags::LATER_INVOKED_CALLABLE, CallableInvocationKind::Later],
            [MetadataFlags::IMMEDIATELY_INVOKED_CALLABLE | MetadataFlags::LATER_INVOKED_CALLABLE, null],
        ];
        foreach ($cases as [$bits, $expected]) {
            $parameter = new ParameterMetadata(
                '$callback',
                $location,
                $location,
                null,
                null,
                null,
                null,
                null,
                [],
                new MetadataFlags($bits | MetadataFlags::VARIADIC),
            );
            self::assertSame($expected, $parameter->callableInvocationKind());
            self::assertTrue($parameter->flags->contains(MetadataFlags::VARIADIC));
        }
    }

    public function testTimingSurvivesMethodParameterWireDecoding(): void
    {
        $cases = [
            [0,                                           null],
            [MetadataFlags::IMMEDIATELY_INVOKED_CALLABLE, CallableInvocationKind::Immediately],
            [MetadataFlags::LATER_INVOKED_CALLABLE,       CallableInvocationKind::Later],
        ];
        foreach ($cases as [$bits, $expected]) {
            $writer = new PayloadWriter();
            $writer->writeBytes('Example');
            $writer->writeBytes('run');
            $writer->writeU8(2); // Method identifier.
            $writer->writeBytes('Example');
            $writer->writeBytes('run');
            $writer->writeU32(1); // One parameter.
            $writer->writeBytes('$callback');
            for ($index = 0; $index < 2; ++$index) {
                $writer->writeBoolean(false); // Location without a file.
                $writer->writeU32(0);
                $writer->writeU32(1);
            }
            for ($index = 0; $index < 5; ++$index) {
                $writer->writeBoolean(false); // Optional type metadata.
            }
            $writer->writeU32(0); // Attributes.
            $writer->writeU64($bits | MetadataFlags::VARIADIC);
            $reader = new PayloadReader($writer->finish());
            $projection = MetadataCodec::readMethodProjection($reader, MethodFields::PARAMETERS);
            $reader->finish();
            self::assertNotNull($projection->parameters);
            self::assertCount(1, $projection->parameters);
            self::assertSame($expected, $projection->parameters[0]->callableInvocationKind());
            self::assertTrue($projection->parameters[0]->flags->contains(MetadataFlags::VARIADIC));
        }
    }
}
