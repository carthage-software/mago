<?php

declare(strict_types=1);

function probe(): int
{
    /**
     * @mago-expect analysis:match-arm-always-true
     * @mago-expect analysis:unreachable-match-arm
     */
    return match ('foo') {
        'foo' => 1,
        'bar' => 2,
    };
}
