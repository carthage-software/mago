<?php

declare(strict_types=1);

/**
 * @return list<array-key>
 */
function array_keys_on_iterator(\ArrayIterator $it): array
{
    /** @mago-expect analysis:invalid-argument */
    return array_keys($it);
}

/**
 * @return list<mixed>
 */
function array_values_on_iterator(\ArrayIterator $it): array
{
    /** @mago-expect analysis:invalid-argument */
    return array_values($it);
}
