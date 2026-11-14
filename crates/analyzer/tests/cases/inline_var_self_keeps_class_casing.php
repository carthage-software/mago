<?php

declare(strict_types=1);

final class Kind
{
    /** @return 'Kind!' */
    public function viaAssignment(mixed $m): string
    {
        /** @var self $s */
        $s = $m;

        return $s::class . '!';
    }

    /** @return 'Kind!' */
    public function viaGlobal(): string
    {
        /** @var self $k */
        global $k;

        return $k::class . '!';
    }
}
