<?php

declare(strict_types=1);

final class ClassesRetA {}

final class ClassesRetB
{
    public function get(): self
    {
        return new ClassesRetA();
    }
}
