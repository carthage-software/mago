<?php

declare(strict_types=1);

class InhSelfUnknown
{
    public static function go(): void
    {
        /** @mago-expect analysis:non-existent-method */
        self::nonexistent();
    }
}
