<?php

declare(strict_types=1);

class InhSelfUnknown
{
    public static function go(): void
    {
        self::nonexistent();
    }
}
