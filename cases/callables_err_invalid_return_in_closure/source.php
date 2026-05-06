<?php

declare(strict_types=1);

$cb = function (): int {
    return 'wrong';
};

echo $cb();
