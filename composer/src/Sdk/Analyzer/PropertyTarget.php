<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\Exception\InvalidArgumentException;

use function strlen;
use function strpos;

/**
 * Selects properties handled by a property type or initialization provider.
 *
 * Exact classes also match their subclasses and implementations.
 * Property names do not include the leading `$`.
 *
 * @api
 */
final class PropertyTarget
{
    /** @var non-empty-string */
    public readonly string $class;

    /** @var non-empty-string */
    public readonly string $property;

    public function __construct(string $class, string $property)
    {
        if ($class === '' || $property === '') {
            throw new InvalidArgumentException('Analyzer property target class and property cannot be empty.');
        }

        if ($property[0] === '$') {
            throw new InvalidArgumentException('Analyzer property target names must not begin with `$`.');
        }

        foreach ([$class, $property] as $pattern) {
            $wildcard = strpos($pattern, '*');
            if ($wildcard !== false && $wildcard !== (strlen($pattern) - 1)) {
                throw new InvalidArgumentException('Analyzer property target wildcards are only allowed at the end.');
            }
        }

        $this->class = $class;
        $this->property = $property;
    }

    public static function exact(string $class, string $property): self
    {
        return new self($class, $property);
    }

    public static function allProperties(string $class): self
    {
        return new self($class, '*');
    }

    public static function anyClass(string $property): self
    {
        return new self('*', $property);
    }
}
