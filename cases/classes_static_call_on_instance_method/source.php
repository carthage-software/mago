<?php

declare(strict_types=1);

final class ClassesStaticOnInstance
{
    public function instance(): int
    {
        return 1;
    }
}

function classesStaticInstance(): mixed
{
    return ClassesStaticOnInstance::instance();
}
