<?php

declare(strict_types=1);

final class ClassesCtorUnknownNamed
{
    public function __construct(public int $value)
    {
    }
}

function classesUnknownNamed(): void
{
    /** @mago-expect analysis:invalid-named-argument */
    $_ = new ClassesCtorUnknownNamed(bogus: 1);
}
