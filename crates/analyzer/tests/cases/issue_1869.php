<?php

declare(strict_types=1);

namespace App;

function test(string $type): void
{
    $msg = '1';
    $type === 't' && $msg = '2';
    echo $msg;
}
