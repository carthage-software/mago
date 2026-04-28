<?php

declare(strict_types=1);

final class ClassesMethodInvalidDef
{
    /** @mago-expect analysis:invalid-parameter-default-value */
    public function take(int $a = 'string'): int
    {
        return $a;
    }
}
