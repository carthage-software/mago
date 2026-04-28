<?php

declare(strict_types=1);

/** @mago-expect analysis:invalid-parameter-default-value */
function bad(int $n = 1.5): int {
    return $n;
}

bad();
