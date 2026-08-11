<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal;

use Mago\Sdk\Exception\ProtocolException;
use Mago\Sdk\Internal\Io\ResourceWriter;
use Mago\Sdk\Internal\Protocol\Frame;
use Mago\Sdk\Internal\Protocol\FrameCodec;
use Mago\Sdk\Internal\Protocol\FrameKind;
use Revolt\EventLoop;
use Revolt\EventLoop\Suspension;
use Throwable;

/**
 * @internal
 * @mago-expect lint:cyclomatic-complexity
 */
final class HostClient
{
    /** @var int<0, max> */
    private int $nextRequestId = 0;

    /**
     * @var array<int<1, max>, array{int<1, max>, Suspension<string>}>
     */
    private array $pending = [];

    public function __construct(
        private readonly FrameCodec $codec,
        private readonly ResourceWriter $writer,
    ) {}

    /** @param int<1, max> $parentId */
    public function request(int $parentId, string $payload): string
    {
        $requestId = ++$this->nextRequestId;
        /** @var Suspension<string> $suspension */
        $suspension = EventLoop::getSuspension();
        $this->pending[$requestId] = [$parentId, $suspension];
        try {
            $this->writer->write($this->codec->encode(Frame::request($requestId, $parentId, $payload)));

            return $suspension->suspend();
        } finally {
            unset($this->pending[$requestId]);
        }
    }

    public function accept(Frame $frame): bool
    {
        if ($frame->kind !== FrameKind::Response) {
            return false;
        }

        $pending = $this->pending[$frame->id] ?? null;
        if ($pending === null) {
            return false;
        }

        [$parentId, $suspension] = $pending;
        unset($this->pending[$frame->id]);
        if ($frame->parentId !== $parentId) {
            $suspension->throw(
                new ProtocolException(
                    "Mago responded to nested request {$frame->id} with the wrong parent identifier.",
                ),
            );

            return true;
        }

        if ($frame->flags === 0) {
            $suspension->resume($frame->payload);

            return true;
        }

        if ($frame->flags === Frame::ERROR_FLAG) {
            $suspension->throw(new ProtocolException($frame->payload));

            return true;
        }

        $suspension->throw(
            new ProtocolException("Nested response {$frame->id} has unsupported flags {$frame->flags}."),
        );

        return true;
    }

    public function fail(Throwable $throwable): void
    {
        $pending = $this->pending;
        $this->pending = [];
        foreach ($pending as [, $suspension]) {
            $suspension->throw($throwable);
        }
    }
}
