<?php

declare(strict_types=1);

final class Magic
{
    /** @var array<string, int> */
    private array $values = [];

    public function __get(string $name): int
    {
        return $this->values[$name] ?? 0;
    }

    public function __set(string $name, int $value): void
    {
        $this->values[$name] = $value;
    }

    /** @param list<mixed> $arguments */
    public function __call(string $name, array $arguments): int
    {
        return strlen($name) + count($arguments);
    }
}

function read(Magic $magic): int
{
    // @mago-expect analysis:non-documented-property
    return $magic->size;
}

function write(Magic $magic): void
{
    // @mago-expect analysis:non-documented-property,invalid-property-assignment-value
    $magic->size = 'large';
}

function call(Magic $magic): int
{
    // @mago-expect analysis:non-documented-method
    return $magic->size();
}
