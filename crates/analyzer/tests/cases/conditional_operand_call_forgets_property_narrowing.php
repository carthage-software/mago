<?php

declare(strict_types=1);

final class Loader
{
    public ?int $value = 1;

    public function load(): bool
    {
        $this->value = null;

        return true;
    }

    public function and_operand(bool $flag): void
    {
        if ($this->value === null) {
            return;
        }

        if ($flag && !$this->load()) {
            return;
        }

        if ($this->value === null) {
            echo 'cleared';
        }
    }

    public function or_operand(bool $flag): void
    {
        if ($this->value === null) {
            return;
        }

        if ($flag || !$this->load()) {
            return;
        }

        if ($this->value === null) {
            echo 'cleared';
        }
    }

    public function ternary_branch(bool $flag): void
    {
        if ($this->value === null) {
            return;
        }

        $loaded = $flag ? $this->load() : false;
        if ($this->value === null) {
            echo 'cleared';
        }

        echo $loaded ? 'loaded' : 'kept';
    }

    public function expression_statement(bool $flag): void
    {
        if ($this->value === null) {
            return;
        }

        $loaded = $flag && $this->load();
        if ($this->value === null) {
            echo 'cleared';
        }

        echo $loaded ? 'loaded' : 'kept';
    }
}
