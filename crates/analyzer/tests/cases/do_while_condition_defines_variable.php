<?php

declare(strict_types=1);

/** @return 0 */
function last(): int
{
    do {
        echo 'again';
    } while ($n = rand(0, 3));

    return $n;
}

function last_line(): false
{
    do {
        echo 'again';
    } while (false !== ($line = fgets(STDIN)));

    return $line;
}
