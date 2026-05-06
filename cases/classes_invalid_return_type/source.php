<?php

declare(strict_types=1);

final class ClassesBadReturnType
{
    public function get(): int
    {
        return 'string';
    }
}
