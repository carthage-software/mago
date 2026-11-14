<?php

declare(strict_types=1);

final class Bag
{
    public function get(): int
    {
        return 1;
    }
}

function f(?Bag $b, bool $force): int
{
    $has = $b !== null;
    $forced = $force && ($has = true);
    if ($forced) {
        echo 'forced';
    }

    if ($has) {
        // @mago-expect analysis:possible-method-access-on-null
        return $b->get();
    }

    return 0;
}
