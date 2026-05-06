<?php

declare(strict_types=1);

final class ClassesMethodInvalidThrow
{
    public function fail(int $code): never
    {
        throw $code;
    }
}
