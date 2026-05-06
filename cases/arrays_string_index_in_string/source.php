<?php

declare(strict_types=1);

function bad_string_index(): mixed
{
    $s = 'hello';
    return $s['foo'];
}
