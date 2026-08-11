<?php

declare(strict_types=1);

namespace Mago\Sdk;

use Mago\Sdk\Exception\InvalidArgumentException;

use function intdiv;

/**
 * The PHP language version selected by Mago for the current operation.
 *
 * @api
 * @mago-expect lint:cyclomatic-complexity
 */
final class PHPVersion
{
    /**
     * @var int<0, 4294967295>
     */
    public readonly int $id;

    /**
     * @param int $id PHP_VERSION_ID-style integer.
     */
    public function __construct(int $id)
    {
        if ($id < 0 || $id > 4_294_967_295) {
            throw new InvalidArgumentException('A PHP version identifier must fit in an unsigned 32-bit integer.');
        }

        $this->id = $id;
    }

    public static function fromParts(int $major, int $minor, int $patch = 0): self
    {
        if ($major < 0 || $major > 429_496 || $minor < 0 || $minor > 99 || $patch < 0 || $patch > 99) {
            throw new InvalidArgumentException('PHP version parts are outside their supported ranges.');
        }

        return new self(($major * 10_000) + ($minor * 100) + $patch);
    }

    public function major(): int
    {
        return intdiv($this->id, 10_000);
    }

    public function minor(): int
    {
        return intdiv($this->id % 10_000, 100);
    }

    public function patch(): int
    {
        return $this->id % 100;
    }

    public function isAtLeast(self $other): bool
    {
        return $this->id >= $other->id;
    }

    public function __toString(): string
    {
        return $this->major() . '.' . $this->minor() . '.' . $this->patch();
    }
}
