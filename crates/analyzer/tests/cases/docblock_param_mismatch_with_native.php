<?php

declare(strict_types=1);

/**
 * @param string $x
 */
function paramMismatchBO(int $x): void // @mago-expect analysis:docblock-type-mismatch
{
    echo $x;
}
