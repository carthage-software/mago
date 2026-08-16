<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Io;

use Mago\Sdk\Exception\ProtocolException;

use function fclose;
use function fwrite;
use function getenv;
use function stream_socket_client;
use function stream_socket_shutdown;
use function strlen;
use function substr;

use const STREAM_CLIENT_CONNECT;
use const STREAM_SHUT_WR;

/**
 * Connects the worker's non-blocking input transport when requested by Mago.
 *
 * @internal
 * @mago-expect lint:cyclomatic-complexity
 */
final class InputTransport
{
    private const ADDRESS_ENVIRONMENT_VARIABLE = 'MAGO_EXTENSION_INPUT_ADDRESS';
    private const TOKEN_ENVIRONMENT_VARIABLE = 'MAGO_EXTENSION_INPUT_TOKEN';
    private const TOKEN_LENGTH = 32;
    private const CONNECT_TIMEOUT_SECONDS = 10.0;

    /**
     * @return resource|null
     */
    public static function connect(): mixed
    {
        $address = getenv(self::ADDRESS_ENVIRONMENT_VARIABLE);
        $token = getenv(self::TOKEN_ENVIRONMENT_VARIABLE);
        if ($address === false && $token === false) {
            return null;
        }

        if ($address === false || $address === '' || $token === false || strlen($token) !== self::TOKEN_LENGTH) {
            throw new ProtocolException('Mago supplied an invalid extension input transport.');
        }

        $errorCode = 0;
        $errorMessage = '';
        $stream = stream_socket_client(
            'tcp://' . $address,
            $errorCode,
            $errorMessage,
            self::CONNECT_TIMEOUT_SECONDS,
            STREAM_CLIENT_CONNECT,
        );

        if ($stream === false) {
            throw new ProtocolException(
                "Unable to connect the extension input transport: [{$errorCode}] {$errorMessage}.",
            );
        }

        $remaining = $token;
        while ($remaining !== '') {
            $written = fwrite($stream, $remaining);
            if ($written === false || $written === 0) {
                fclose($stream);
                throw new ProtocolException('Unable to authenticate the extension input transport.');
            }

            $remaining = substr($remaining, $written);
        }

        if (!stream_socket_shutdown($stream, STREAM_SHUT_WR)) {
            fclose($stream);
            throw new ProtocolException('Unable to finalize the extension input transport handshake.');
        }

        return $stream;
    }
}
