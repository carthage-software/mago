<?php

declare(strict_types=1);

function example(int $x): bool {
    /** @mago-expect analysis:redundant-type-comparison */
    return is_int($x);
}
