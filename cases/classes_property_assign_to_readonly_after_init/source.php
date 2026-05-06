<?php

declare(strict_types=1);

final class ClassesReadOnlyTwiceWrite
{
    public readonly int $value;

    public function __construct()
    {
        $this->value = 1;
    }

    public function rewrite(): void
    {
        $this->value = 2;
    }
}
