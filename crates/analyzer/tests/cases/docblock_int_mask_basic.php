<?php

declare(strict_types=1);

/**
 * @param int-mask<1, 2, 4> $flags
 */
function set_flags(int $flags): void
{
    echo $flags;
}

set_flags(0);
set_flags(1);
set_flags(7);
/** @mago-expect analysis:invalid-argument */
set_flags(8);
