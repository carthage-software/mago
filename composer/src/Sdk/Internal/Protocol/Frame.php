<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Protocol;

/**
 * @internal
 */
final class Frame
{
    public const ERROR_FLAG = 1;

    /**
     * @param int<0, 255> $flags
     * @param int<0, max> $id
     * @param int<0, max> $parentId
     */
    public function __construct(
        public readonly FrameKind $kind,
        public readonly int $flags,
        public readonly int $id,
        public readonly int $parentId,
        public readonly string $payload,
    ) {}

    /**
     * @param int<0, max> $id
     */
    public static function response(int $id, string $payload): self
    {
        return new self(FrameKind::Response, 0, $id, 0, $payload);
    }

    /**
     * @param int<0, max> $id
     */
    public static function error(int $id, string $payload): self
    {
        return new self(FrameKind::Response, self::ERROR_FLAG, $id, 0, $payload);
    }

    /**
     * @param int<0, max> $id
     * @param int<1, max> $parentId
     */
    public static function request(int $id, int $parentId, string $payload): self
    {
        return new self(FrameKind::Request, 0, $id, $parentId, $payload);
    }
}
