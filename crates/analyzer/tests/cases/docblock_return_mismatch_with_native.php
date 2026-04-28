<?php

declare(strict_types=1);

/**
 * @return int
 */
function returnMismatchBP(): string // @mago-expect analysis:docblock-type-mismatch
{
    return 'x'; // @mago-expect analysis:invalid-return-statement
}
