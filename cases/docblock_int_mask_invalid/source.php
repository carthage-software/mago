<?php

declare(strict_types=1);

/**
 * @param int-mask<1, 2, 4> $flags
 */
function flagsO(int $flags): void
{
    echo $flags;
}

flagsO(8);
flagsO(15);
