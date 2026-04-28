<?php

declare(strict_types=1);

function bad_string_index(): mixed
{
    $s = 'hello';
    // @mago-expect analysis:invalid-array-index
    return $s['foo'];
}
