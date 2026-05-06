<?php

declare(strict_types=1);

function takesString(string $s): string
{
    return $s;
}

$a = 'a:' . true;
takesString($a);
$b = 'b:' . false;
takesString($b);
