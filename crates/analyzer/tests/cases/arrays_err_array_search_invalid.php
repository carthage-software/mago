<?php

declare(strict_types=1);

function bad(int $haystack): mixed
{
    // @mago-expect analysis:invalid-argument
    return array_search(1, $haystack, true);
}
