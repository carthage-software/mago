<?php

declare(strict_types=1);

/** @param non-empty-string $s */
function takes_non_empty(string $s): void
{
    echo $s;
}

function probe(string $name): void
{
    $message = <<<EOT
Hello, {$name}!
EOT;

    takes_non_empty($message);
}
