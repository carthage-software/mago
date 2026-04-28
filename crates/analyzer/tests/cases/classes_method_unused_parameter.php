<?php

declare(strict_types=1);

final class ClassesUnusedMethodParam
{
    public function compute(int $unused): int
    {
        unset($unused);
        return 0;
    }
}

echo (new ClassesUnusedMethodParam())->compute(1);
