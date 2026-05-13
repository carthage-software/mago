<?php

declare(strict_types=1);

class StaticLocalArrayCoalesce
{
    public function __construct(public string $table) {}

    public static function create(string $table): self
    {
        static $instances = [];

        return $instances[$table] ??= new self($table);
    }
}
