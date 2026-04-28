<?php

declare(strict_types=1);

/** @param non-empty-string $s */
function takes_non_empty(string $s): void
{
    echo $s;
}

function probe(): void
{
    $s = <<<'EOT'
literal $not_interpolated content
EOT;

    takes_non_empty($s);
}
