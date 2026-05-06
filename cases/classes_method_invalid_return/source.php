<?php

declare(strict_types=1);

final class ClassesMethodInvalidReturn
{
    public function get(): string
    {
        return 42;
    }
}
