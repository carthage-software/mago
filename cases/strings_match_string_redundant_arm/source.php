<?php

declare(strict_types=1);

function probe(): int
{
    /**
     */
    return match ('foo') {
        'foo' => 1,
        'bar' => 2,
    };
}
