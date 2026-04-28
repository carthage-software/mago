<?php

declare(strict_types=1);

final class ClassesVoidMisuse
{
    /** @mago-expect analysis:invalid-return-statement */
    public function nothing(): void
    {
        return 5;
    }
}
