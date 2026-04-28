<?php

declare(strict_types=1);

/** @param int-mask<1, 2, 4> $f */
function flagsBZ(int $f): void
{
    echo $f;
}

/** @mago-expect analysis:invalid-argument */
flagsBZ(-1);
