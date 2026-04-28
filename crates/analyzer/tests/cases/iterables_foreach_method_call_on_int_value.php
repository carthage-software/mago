<?php

declare(strict_types=1);

/**
 * @param list<int> $items
 */
function bad(array $items): void
{
    foreach ($items as $v) {
        /** @mago-expect analysis:invalid-method-access */
        $v->whatever();
    }
}
