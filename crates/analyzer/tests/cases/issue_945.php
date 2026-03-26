<?php

declare(strict_types=1);

trait TraitExample
{
    private readonly mixed $value;

    public function __construct(mixed $value)
    {
        self::guard($value);
        $this->value = $value;
    }

    private static function guard(mixed $value): void
    {
        assert($value !== null, 'Value must not be null.');
    }
}

final class ClassExample
{
    use TraitExample;

    private static function guard(mixed $value): void
    {
        assert(is_string($value), 'Value must be a string.');
    }
}
