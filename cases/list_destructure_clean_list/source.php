<?php

declare(strict_types=1);

/** @return list<string> */
function t(): array
{
    return ['a', 'b'];
}

[$a, $b] = t();
echo $a;
echo $b;
