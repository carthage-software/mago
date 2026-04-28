<?php

declare(strict_types=1);

final class ClassesReturnsSelf
{
    public int $count = 0;

    public function bump(): self
    {
        $this->count++;
        return $this;
    }
}

(new ClassesReturnsSelf())->bump()->bump();
