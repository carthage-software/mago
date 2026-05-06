<?php

declare(strict_types=1);

final class ClassesArgTooSpec
{
    public function take(string $name): void
    {
        unset($name);
    }
}

function classesArgTooSpec(int $n): void
{
    (new ClassesArgTooSpec())->take($n);
}
