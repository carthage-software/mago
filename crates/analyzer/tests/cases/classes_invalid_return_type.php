<?php

declare(strict_types=1);

final class ClassesBadReturnType
{
    /** @mago-expect analysis:invalid-return-statement */
    public function get(): int
    {
        return 'string';
    }
}
