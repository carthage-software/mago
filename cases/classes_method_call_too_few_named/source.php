<?php

declare(strict_types=1);

final class ClassesMcallFewNamed
{
    public function take(int $a, int $b): int
    {
        return $a + $b;
    }
}

function classesMcallFewNamed(): void
{
    (new ClassesMcallFewNamed())->take(a: 1);
}
