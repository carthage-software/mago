<?php

declare(strict_types=1);

final class UnsetMemoizedHolder
{
    /** @var array<string, int> */
    private array $map = ['a' => 1];

    public ?string $name = 'name';

    /** @mutation-free */
    private function hasA(): bool
    {
        return isset($this->map['a']);
    }

    /** @mutation-free */
    private function hasName(): bool
    {
        return isset($this->name);
    }

    public function offset(): string
    {
        if ($this->hasA()) {
            unset($this->map['a']);
            if ($this->hasA()) {
                return 'still';
            }

            return 'gone';
        }

        return 'none';
    }

    public function property(): string
    {
        if ($this->hasName()) {
            unset($this->name);
            if ($this->hasName()) {
                return 'still';
            }

            return 'gone';
        }

        return 'none';
    }
}
