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
    /** @mago-expect analysis:too-few-arguments */
    (new ClassesTooFewArgs())->take(1);
}
