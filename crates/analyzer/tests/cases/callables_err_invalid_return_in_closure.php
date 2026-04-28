<?php

declare(strict_types=1);

$cb = function (): int {
    /** @mago-expect analysis:invalid-return-statement */
    return 'wrong';
};

echo $cb();
