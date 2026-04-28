<?php

declare(strict_types=1);

final class ClassesDebugInfoMagic
{
    public int $value = 0;

    /** @return array<string, int> */
    public function __debugInfo(): array
    {
        return ['value' => $this->value];
    }
}

var_dump(new ClassesDebugInfoMagic());
