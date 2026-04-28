<?php

declare(strict_types=1);

final class ClassesArgNullableReq
{
    public function take(string $value): void
    {
        unset($value);
    }
}

function classesArgNullableReq(null|string $value): void
{
    /** @mago-expect analysis:possibly-null-argument */
    (new ClassesArgNullableReq())->take($value);
}
