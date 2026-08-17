<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\Exception\InvalidArgumentException;

use function strlen;
use function strpos;

/**
 * Selects class-likes handled by a class-oriented analyzer provider.
 *
 * Exact classes also match their subclasses and implementations.
 *
 * @api
 */
final class ClassTarget
{
    /** @var non-empty-string */
    public readonly string $class;

    public function __construct(string $class)
    {
        if ($class === '') {
            throw new InvalidArgumentException('Analyzer class target cannot be empty.');
        }

        $wildcard = strpos($class, '*');
        if ($wildcard !== false && $wildcard !== (strlen($class) - 1)) {
            throw new InvalidArgumentException('Analyzer class target wildcards are only allowed at the end.');
        }

        $this->class = $class;
    }

    public static function exact(string $class): self
    {
        return new self($class);
    }

    public static function any(): self
    {
        return new self('*');
    }
}
