<?php

declare(strict_types=1);

function bad_index(int $n): mixed
{
    // @mago-expect analysis:invalid-array-access
    return $n[0];
}
