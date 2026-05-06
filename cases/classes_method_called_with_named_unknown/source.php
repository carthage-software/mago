<?php

declare(strict_types=1);

final class ClassesMcallUnknownNamed
{
    public function take(int $value): int
    {
        return $value;
    }
}

function classesMcallUnknownNamed(): void
{
    (new ClassesMcallUnknownNamed())->take(bogus: 1);
}
