<?php

declare(strict_types=1);

class NodeA
{
    public function __construct(
        public readonly ?NodeA $parent = null,
    ) {}

    public function find(string $name): ?string
    {
        return $this->parent?->find($name);
    }

    public function set(string $name, string $value): void
    {
        if ($this->parent !== null && $this->parent->find($name) !== null) {
            $this->parent->set($name, $value);

            return;
        }
    }
}

readonly class NodeB
{
    public function __construct(
        public ?NodeB $parent = null,
    ) {}

    public function find(string $name): ?string
    {
        return $this->parent?->find($name);
    }

    public function set(string $name, string $value): void
    {
        if ($this->parent !== null && $this->parent->find($name) !== null) {
            $this->parent->set($name, $value);

            return;
        }
    }
}
