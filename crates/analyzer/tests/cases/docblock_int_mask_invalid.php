<?php

declare(strict_types=1);

/**
 * @param int-mask<1, 2, 4> $flags
 */
function flagsO(int $flags): void
{
    echo $flags;
}

/** @mago-expect analysis:invalid-argument */
flagsO(8);
/** @mago-expect analysis:invalid-argument */
flagsO(15);
