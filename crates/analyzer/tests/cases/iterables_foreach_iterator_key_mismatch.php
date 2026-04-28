<?php

declare(strict_types=1);

function take_int(int $_n): void
{
}

/**
 * @param Iterator<string, int> $it
 */
function iterate_iter(Iterator $it): void
{
    foreach ($it as $key => $_value) {
        /** @mago-expect analysis:invalid-argument */
        take_int($key);
    }
}
