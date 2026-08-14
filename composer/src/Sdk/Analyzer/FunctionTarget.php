<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\Exception\InvalidArgumentException;

/**
 * @api
 */
final class FunctionTarget
{
    /** @var non-empty-string */
    public readonly string $value;

    private function __construct(
        public readonly FunctionTargetKind $kind,
        string $value,
    ) {
        if ($value === '') {
            throw new InvalidArgumentException('An analyzer function target cannot be empty.');
        }

        $this->value = $value;
    }

    public static function exact(string $function): self
    {
        return new self(FunctionTargetKind::Exact, $function);
    }

    public static function prefix(string $prefix): self
    {
        return new self(FunctionTargetKind::Prefix, $prefix);
    }

    public static function namespace(string $namespace): self
    {
        return new self(FunctionTargetKind::Namespace, $namespace);
    }
}
