<?php

declare(strict_types=1);

function bad(int $n): string
{
    // @mago-expect analysis:invalid-argument
    return implode(',', $n);
}
