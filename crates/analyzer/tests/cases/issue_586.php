<?php

declare(strict_types=1);

function gar(?string $y): void
{
    $x = ['' => 2];

    // @mago-expect analysis:possibly-null-array-index
    if (isset($x[$y])) {
        echo $x[$y], PHP_EOL;
        if ($y === null) {
            echo 123, PHP_EOL;
        }
    }
}

gar(null);
