<?php

declare(strict_types=1);

/** @param non-empty-literal-string $s */
function takes_nelit(string $s): void
{
    echo $s;
}

takes_nelit('hello');
