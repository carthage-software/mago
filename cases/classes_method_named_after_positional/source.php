<?php

declare(strict_types=1);

final class ClassesMcallNamedAfterPos
{
    public function take(int $a, int $b): int
    {
        return $a + $b;
    }
}

function classesMcallNamedAfterPos(): void
{
    (new ClassesMcallNamedAfterPos())->take(1, a: 2);
}
