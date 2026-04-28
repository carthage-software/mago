<?php

declare(strict_types=1);

/**
 * @param list<int> $xs
 */
function bad(array $xs): void
{
    // @mago-expect analysis:invalid-argument
    array_walk($xs, 42);
}
