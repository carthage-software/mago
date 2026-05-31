<?php

declare(strict_types=1);

class Resource
{
    public function id(): int
    {
        return 1;
    }
}

/**
 * @param Resource $r
 */
function takes($r): int
{
    return $r->id();
}
