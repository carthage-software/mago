<?php

declare(strict_types=1);

/** @param non-empty-literal-string $s */
function takes_nelit(string $s): void
{
    echo $s;
}

/** @mago-expect analysis:invalid-argument */
takes_nelit('');
