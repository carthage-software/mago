<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\Exception\InvalidArgumentException;

use function strlen;
use function strpos;

/**
 * Selects method calls handled by a return-type or callable-signature provider.
 *
 * Exact classes also match their subclasses and implementations.
 *
 * @api
 */
final class MethodTarget
{
    /** @var non-empty-string */
    public readonly string $class;

    /** @var non-empty-string */
    public readonly string $method;

    public function __construct(string $class, string $method)
    {
        if ($class === '' || $method === '') {
            throw new InvalidArgumentException('Analyzer method target class and method cannot be empty.');
        }

        foreach ([$class, $method] as $pattern) {
            $wildcard = strpos($pattern, '*');
            if ($wildcard !== false && $wildcard !== (strlen($pattern) - 1)) {
                throw new InvalidArgumentException('Analyzer method target wildcards are only allowed at the end.');
            }
        }

        $this->class = $class;
        $this->method = $method;
    }

    public static function exact(string $class, string $method): self
    {
        return new self($class, $method);
    }

    public static function allMethods(string $class): self
    {
        return new self($class, '*');
    }

    public static function anyClass(string $method): self
    {
        return new self('*', $method);
    }
}
