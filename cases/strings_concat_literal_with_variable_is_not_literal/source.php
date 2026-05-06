<?php

declare(strict_types=1);

/** @param literal-string $s */
function takes_literal(string $s): void
{
    echo $s;
}

function probe(string $tail): void
{
    takes_literal('prefix_' . $tail);
}
