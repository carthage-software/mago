<?php

declare(strict_types=1);

final class ClassesVoidMethod
{
    public function noop(): void
    {
        return;
    }
}

(new ClassesVoidMethod())->noop();
