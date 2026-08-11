<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Io;

use Mago\Sdk\Exception\InvalidArgumentException;
use Mago\Sdk\Exception\ProtocolException;
use Revolt\EventLoop;
use Revolt\EventLoop\Suspension;

use function feof;
use function fwrite;
use function is_resource;
use function stream_set_blocking;
use function stream_set_write_buffer;
use function strlen;
use function substr;

/**
 * A serialized, non-blocking writer specialized for Mago's worker pipe.
 *
 * @internal
 * @mago-expect lint:cyclomatic-complexity
 */
final class ResourceWriter
{
    /**
     * @var resource
     */
    private readonly mixed $stream;

    private string $watcher = '';

    /**
     * @var null|Suspension<mixed>
     */
    private ?Suspension $suspension = null;

    /**
     * @var array<int<0, max>, string>
     */
    private array $queue = [];

    private bool $writing = false;

    /**
     * @param resource $stream
     */
    public function __construct(mixed $stream)
    {
        if (!is_resource($stream)) {
            throw new InvalidArgumentException('The worker output must be an open stream resource.');
        }

        if (!stream_set_blocking($stream, false)) {
            throw new InvalidArgumentException('The worker output stream cannot be made non-blocking.');
        }

        stream_set_write_buffer($stream, 0);
        $this->stream = $stream;
        $this->watcher = EventLoop::onWritable($stream, function (): void {
            EventLoop::disable($this->watcher);
            $suspension = $this->suspension;
            $this->suspension = null;
            $suspension?->resume();
        });
        EventLoop::disable($this->watcher);
    }

    /**
     * Queue bytes and drain them in frame order.
     *
     * @mago-expect lint:no-isset
     */
    public function write(string $bytes): void
    {
        if ($bytes === '') {
            return;
        }

        if ($this->writing) {
            $this->queue[] = $bytes;
            return;
        }

        $this->writing = true;
        $index = 0;
        try {
            $this->writeAll($bytes);
            while (isset($this->queue[$index])) {
                $bytes = $this->queue[$index];
                unset($this->queue[$index]);
                ++$index;
                $this->writeAll($bytes);
            }
        } finally {
            $this->queue = [];
            $this->writing = false;
        }
    }

    public function close(): void
    {
        EventLoop::cancel($this->watcher);
    }

    private function writeAll(string $bytes): void
    {
        $length = strlen($bytes);
        while ($bytes !== '') {
            $written = fwrite($this->stream, $bytes);
            if ($written === false) {
                throw new ProtocolException('Unable to write to the Mago worker output stream.');
            }

            if ($written > 0) {
                if ($written === $length) {
                    return;
                }

                $bytes = substr($bytes, $written);
                $length -= $written;
                continue;
            }

            if (feof($this->stream)) {
                throw new ProtocolException('The Mago worker output stream was closed while writing a response.');
            }

            $this->awaitWritable();
        }
    }

    private function awaitWritable(): void
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
