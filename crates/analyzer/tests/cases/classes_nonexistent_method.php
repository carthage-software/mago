<?php

declare(strict_types=1);

final class ClassesNonExistMethod
{
    public function ping(): string
    {
        return 'pong';
    }
}

function classesNonExistMethod(ClassesNonExistMethod $obj): mixed
{
    /** @mago-expect analysis:non-existent-method */
    return $obj->bogus();
}
