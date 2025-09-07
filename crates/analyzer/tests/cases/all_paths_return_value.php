<?php

declare(strict_types=1);

/**
 * @mago-expect analysis:all-paths-must-return
 */
function x(bool $x): int
{
    if($x === true) {
        return 15;
    }
}

