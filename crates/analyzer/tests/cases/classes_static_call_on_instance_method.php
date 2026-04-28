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
    /** @mago-expect analysis:invalid-static-method-access */
    return ClassesStaticOnInstance::instance();
}
