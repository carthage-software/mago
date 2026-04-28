<?php

declare(strict_types=1);

function bad(int $n): void
{
    // @mago-expect analysis:invalid-argument
    sort($n);
}
