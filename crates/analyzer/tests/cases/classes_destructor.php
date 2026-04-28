<?php

declare(strict_types=1);

final class ClassesWithDestructor
{
    public function __destruct()
    {
    }
}

new ClassesWithDestructor();
