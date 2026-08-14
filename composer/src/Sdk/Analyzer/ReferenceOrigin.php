<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\Analyzer\Metadata\MemberIdentifier;
use Mago\Sdk\Exception\InvalidArgumentException;

/**
 * A symbol, class-like member, or file from which a reference originates.
 *
 * @api
 */
final class ReferenceOrigin
{
    private function __construct(
        public readonly string|MemberIdentifier|null $symbol,
        public readonly ?string $file,
    ) {}

    public static function symbol(string|MemberIdentifier $symbol): self
    {
        if ($symbol === '') {
            throw new InvalidArgumentException('A reference source symbol cannot be empty.');
        }

        return new self($symbol, null);
    }

    public static function file(string $file): self
    {
        if ($file === '') {
            throw new InvalidArgumentException('A reference source file cannot be empty.');
        }

        return new self(null, $file);
    }

    public function isFile(): bool
    {
        return $this->file !== null;
    }
}
