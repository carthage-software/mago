<?php

declare(strict_types=1);

/**
 * @param Iterator<string, int> $it
 * @return array<string, int>
 */
function collect(Iterator $it): array
{
    return iterator_to_array($it);
}
