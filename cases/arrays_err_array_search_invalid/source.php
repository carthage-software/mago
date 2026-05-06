<?php

declare(strict_types=1);

function bad(int $haystack): mixed
{
    return array_search(1, $haystack, true);
}
