<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Unit\Io;

use Mago\Sdk\Internal\Io\InputTransport;
use Mago\Sdk\Internal\Io\ResourceReader;
use PHPUnit\Framework\TestCase;
use RuntimeException;

use function fclose;
use function fwrite;
use function putenv;
use function str_repeat;
use function stream_socket_accept;
use function stream_socket_get_name;
use function stream_socket_server;

final class InputTransportTest extends TestCase
{
    public function testConnectsAndAuthenticatesInputStream(): void
    {
        $errorCode = 0;
        $errorMessage = '';
        $server = stream_socket_server('tcp://127.0.0.1:0', $errorCode, $errorMessage);
        if ($server === false) {
            throw new RuntimeException("Unable to create input transport test server: [{$errorCode}] {$errorMessage}.");
        }

        $address = stream_socket_get_name($server, false);
        if ($address === false) {
            fclose($server);
            throw new RuntimeException('Unable to resolve the input transport test server address.');
        }

        $token = str_repeat('a', 32);
        putenv('MAGO_EXTENSION_INPUT_ADDRESS=' . $address);
        putenv('MAGO_EXTENSION_INPUT_TOKEN=' . $token);
        try {
            $input = InputTransport::connect();
            self::assertIsResource($input);
            $peer = stream_socket_accept($server, 1.0);
            if ($peer === false) {
                throw new RuntimeException('Unable to accept the input transport test connection.');
            }

            $peerReader = new ResourceReader($peer);
            self::assertSame($token, $peerReader->readExactly(32));
            $peerReader->close();
            fwrite($peer, 'ready');
            $reader = new ResourceReader($input);
            self::assertSame('ready', $reader->readExactly(5));
            $reader->close();
            fclose($input);
            fclose($peer);
        } finally {
            putenv('MAGO_EXTENSION_INPUT_ADDRESS');
            putenv('MAGO_EXTENSION_INPUT_TOKEN');
            fclose($server);
        }
    }
}
