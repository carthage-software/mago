<?php

declare(strict_types=1);

function bad(string $s): bool
{
    return in_array(42, $s);
}
