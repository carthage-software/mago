<?php

declare(strict_types=1);

final class ClassesMethodNullableRet
{
    /**
     */
    public function get(?string $value): string
    {
        return $value;
    }
}
