<?php

declare(strict_types=1);

final class ClassesMethodChainTyped
{
    public int $count = 0;

    public function bump(): self
    {
        $this->count++;
        return $this;
    }

    public function get(): int
    {
        return $this->count;
    }
}

echo (new ClassesMethodChainTyped())->bump()->bump()->bump()->get();
