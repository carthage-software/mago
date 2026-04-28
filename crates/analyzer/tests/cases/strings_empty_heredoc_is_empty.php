<?php

declare(strict_types=1);

/** @param non-empty-string $s */
function takes_non_empty(string $s): void
{
    echo $s;
}

function probe(): void
{
    $empty = <<<EOT
EOT;

    /** @mago-expect analysis:invalid-argument */
    takes_non_empty($empty);
}
