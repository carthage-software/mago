<?php

declare(strict_types=1);

function example(float $x): bool {
    /** @mago-expect analysis:redundant-type-comparison */
    return is_float($x);
}
