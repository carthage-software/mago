<?php

declare(strict_types=1);

function bad(string $s): bool
{
    // @mago-expect analysis:invalid-argument
    return in_array(42, $s);
}
