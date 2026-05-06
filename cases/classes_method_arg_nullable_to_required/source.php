<?php

declare(strict_types=1);

final class ClassesArgNullableReq
{
    public function take(string $value): void
    {
        unset($value);
    }
}

function classesArgNullableReq(?string $value): void
{
    (new ClassesArgNullableReq())->take($value);
}
