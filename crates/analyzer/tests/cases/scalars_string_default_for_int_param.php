<?php

declare(strict_types=1);

/** @mago-expect analysis:invalid-parameter-default-value */
function bad(int $n = 'oops'): int {
    return $n;
}

bad();
