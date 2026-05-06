<?php

declare(strict_types=1);

final class ClassesMethodFalsableRet
{
    /**
     */
    public function get(false|string $value): string
    {
        return $value;
    }
}
