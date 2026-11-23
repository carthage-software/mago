<?php

declare(strict_types=1);

function empty_count_keeps_countable_objects(mixed $items): bool
{
    /** @mago-expect analysis:mixed-argument */
    return !$items || count($items) === 0;
}
