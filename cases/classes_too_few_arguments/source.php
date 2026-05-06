<?php

declare(strict_types=1);

final class ClassesTooFewArgs
{
    public function take(int $a, int $b): int
    {
        return $a + $b;
    }
}

function classesTooFew(): void
{
    (new ClassesTooFewArgs())->take(1);
}
