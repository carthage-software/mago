<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Unit\Protocol;

use Mago\Sdk\Internal\Io\ResourceReader;
use Mago\Sdk\Internal\Protocol\Frame;
use Mago\Sdk\Internal\Protocol\FrameCodec;
use PHPUnit\Framework\TestCase;
use RuntimeException;

use function fclose;
use function fwrite;
use function stream_socket_pair;

use const STREAM_IPPROTO_IP;
use const STREAM_PF_UNIX;
use const STREAM_SOCK_STREAM;

final class FrameCodecTest extends TestCase
{
    public function testResponseFrameRoundTrips(): void
    {
        $codec = new FrameCodec(1024);
        $encoded = $codec->encode(Frame::response(42, "payload\0"));
        $streams = stream_socket_pair(STREAM_PF_UNIX, STREAM_SOCK_STREAM, STREAM_IPPROTO_IP);
        if ($streams === false) {
            throw new RuntimeException('Unable to create frame test streams.');
        }

        [$input, $output] = $streams;
        fwrite($output, $encoded);
        fclose($output);
        $reader = new ResourceReader($input);
        $frame = $codec->read($reader);
        $reader->close();
        fclose($input);

        self::assertNotNull($frame);
        self::assertSame(42, $frame->id);
        self::assertSame("payload\0", $frame->payload);
    }
}
