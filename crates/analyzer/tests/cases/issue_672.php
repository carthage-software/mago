<?php

declare(strict_types=1);

class File
{
    public function foo(mixed $data): mixed
    {
        if ($data instanceof Foo) {
            return $data->bar[0]?->value;
        }

        return null;
    }
}

class Foo
{
    /** @var list<Bar> */
    public array $bar = [];
}

class Bar
{
    public null|string $value = null;
}
