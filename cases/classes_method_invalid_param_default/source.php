<?php

declare(strict_types=1);

final class ClassesMethodInvalidDef
{
    public function take(int $a = 'string'): int
    {
        return $a;
    }
}
