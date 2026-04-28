<?php

declare(strict_types=1);

/**
 * @return negative-int
 */
function bad(): int {
    /** @mago-expect analysis:invalid-return-statement */
    return 5;
}
