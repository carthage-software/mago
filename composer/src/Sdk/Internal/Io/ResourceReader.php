<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Io;

use Mago\Sdk\Exception\InvalidArgumentException;
use Mago\Sdk\Exception\ProtocolException;
use Revolt\EventLoop;
use Revolt\EventLoop\Suspension;

use function feof;
use function fread;
use function is_resource;
use function stream_set_blocking;
use function stream_set_read_buffer;
use function strlen;
use function substr;

/**
 * A buffered, non-blocking reader specialized for Mago's worker pipe.
 *
 * @internal
 * @mago-expect lint:cyclomatic-complexity
 */
final class ResourceReader
{
    private const READ_SIZE = 65_536;

    /**
     * @var resource
     */
    private readonly mixed $stream;

    private string $watcher = '';

    /**
     * @var null|Suspension<mixed>
     */
    private ?Suspension $suspension = null;

    private string $buffer = '';

    private int $bufferLength = 0;

    /**
     * @param resource $stream
     */
    public function __construct(mixed $stream)
    {
        if (!is_resource($stream)) {
            throw new InvalidArgumentException('The worker input must be an open stream resource.');
        }

        if (!stream_set_blocking($stream, false)) {
            throw new InvalidArgumentException('The worker input stream cannot be made non-blocking.');
        }

        stream_set_read_buffer($stream, 0);
        $this->stream = $stream;
        $this->watcher = EventLoop::onReadable($stream, function (): void {
            EventLoop::disable($this->watcher);
            $suspension = $this->suspension;
            $this->suspension = null;
            $suspension?->resume();
        });
        EventLoop::disable($this->watcher);
    }

    /**
     * Read exactly the requested number of bytes.
     *
     * Returns null only for a clean EOF before any bytes of the next value.
     *
     * @param int<0, max> $length
     * @mago-expect lint:halstead
     */
    public function readExactly(int $length): ?string
    {
        if ($length === 0) {
            return '';
        }

        while ($this->bufferLength < $length) {
            $chunk = fread($this->stream, self::READ_SIZE);
            if ($chunk === false) {
                throw new ProtocolException('Unable to read from the Mago worker input stream.');
            }

            if ($chunk !== '') {
                $this->buffer .= $chunk;
                $this->bufferLength += strlen($chunk);
                continue;
            }

            if (feof($this->stream)) {
                if ($this->bufferLength === 0) {
                    return null;
                }

                throw new ProtocolException(
                    "The Mago worker input ended with {$this->bufferLength} buffered bytes while reading {$length}.",
                );
            }

            $this->awaitReadable();
        }

        if ($this->bufferLength === $length) {
            $bytes = $this->buffer;
            $this->buffer = '';
            $this->bufferLength = 0;

            return $bytes;
        }

        $bytes = substr($this->buffer, 0, $length);
        $this->buffer = substr($this->buffer, $length);
        $this->bufferLength -= $length;

        return $bytes;
    }

    public function close(): void
    {
        EventLoop::cancel($this->watcher);
    }

    private function awaitReadable(): void
    {
        $suspension = EventLoop::getSuspension();
        $this->suspension = $suspension;
        EventLoop::enable($this->watcher);
        try {
            $suspension->suspend();
        } finally {
            $this->suspension = null;
            EventLoop::disable($this->watcher);
        }
    }
}
