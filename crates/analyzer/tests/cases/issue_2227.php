<?php

declare(strict_types=1);

final class WeakMapHolder
{
    /** @var WeakMap<object, string> */
    private readonly WeakMap $map;

    public function __construct()
    {
        $this->map = new WeakMap();
    }

    public function set(object $key, string $value): void
    {
        $this->map[$key] = $value;
    }
}

final class ArrayObjectHolder
{
    /** @var ArrayObject<string, string> */
    private readonly ArrayObject $map;

    public function __construct()
    {
        $this->map = new ArrayObject();
    }

    public function set(string $key, string $value): void
    {
        $this->map[$key] = $value;
    }
}

final class ArrayHolder
{
    /** @param array<string, string> $values */
    public function __construct(private readonly array $values)
    {
    }

    public function set(string $key, string $value): void
    {
        // @mago-expect analysis:invalid-property-write
        $this->values[$key] = $value;
    }
}
