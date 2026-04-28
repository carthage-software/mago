<?php

declare(strict_types=1);

function example(string $x): bool {
    /** @mago-expect analysis:impossible-type-comparison */
    return is_int($x);
}
