<?php

declare(strict_types=1);

/** @param int-mask<1, 2, 4> $f */
function flagsBY(int $f): void
{
    echo $f;
}

flagsBY(0);
/** @mago-expect analysis:invalid-argument */
flagsBY(16);
