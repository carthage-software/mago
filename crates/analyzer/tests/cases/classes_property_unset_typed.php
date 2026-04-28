<?php

declare(strict_types=1);

final class ClassesPropUnset
{
    public int $count = 0;

    public function reset(): void
    {
        unset($this->count);
    }
}

(new ClassesPropUnset())->reset();
