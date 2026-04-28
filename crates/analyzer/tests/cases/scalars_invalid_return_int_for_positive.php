<?php

declare(strict_types=1);

/**
 * @return positive-int
 */
function bad(): int {
    /** @mago-expect analysis:invalid-return-statement */
    return 0;
}
