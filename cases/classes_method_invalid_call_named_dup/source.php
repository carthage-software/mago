<?php

declare(strict_types=1);

final class ClassesMcallDupNamed
{
    public function take(int $value): int
    {
        return $value;
    }
}

function classesMcallDup(): void
{
    (new ClassesMcallDupNamed())->take(value: 1, value: 2);
}
