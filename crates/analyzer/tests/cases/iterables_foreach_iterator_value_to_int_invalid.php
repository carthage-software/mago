<?php

declare(strict_types=1);

function take_int(int $_n): void
{
}

/**
 * @param Iterator<int, string> $it
 */
function bad(Iterator $it): void
{
    foreach ($it as $v) {
        /** @mago-expect analysis:invalid-argument */
        take_int($v);
    }
}
