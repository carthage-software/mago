<?php

declare(strict_types=1);

/**
 * @deprecated Use new_func() instead.
 */
function oldFuncBB(): int
{
    return 1;
}

/** @mago-expect analysis:deprecated-function */
echo oldFuncBB();
