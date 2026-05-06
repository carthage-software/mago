<?php

declare(strict_types=1);

/** @return array<int<-5, 5>, string> */
function t(): array
{
    return [0 => 'a'];
}

/**
 */
[$a, $b] = t();
echo $a;
echo $b;
