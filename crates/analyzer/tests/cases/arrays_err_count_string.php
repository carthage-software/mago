<?php

declare(strict_types=1);

function bad(string $s): int
{
    // @mago-expect analysis:invalid-argument
    return count($s);
}
