<?php

declare(strict_types=1);

$cb = function (): string {
    /** @mago-expect analysis:invalid-return-statement */
    return 1;
};

echo $cb();
