<?php

declare(strict_types=1);

function bad(int $a): array
{
    // @mago-expect analysis:invalid-argument
    return array_merge([1, 2], $a);
}
