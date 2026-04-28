<?php

declare(strict_types=1);

/** @param literal-string $s */
function takes_literal(string $s): void
{
    echo $s;
}

function probe(string $tail): void
{
    /** @mago-expect analysis:possibly-invalid-argument */
    takes_literal('prefix_' . $tail);
}
