<?php

declare(strict_types=1);

/** @param 1|2 $a */
function foo(int $a): void
{
    /** @mago-expect analysis:redundant-comparison */
    /** @mago-expect analysis:impossible-condition */
    if ($a < 0) {
        echo '<0';
    }
}
