<?php

declare(strict_types=1);

function bad(string $s): array
{
    // @mago-expect analysis:invalid-argument
    return array_values($s);
}
