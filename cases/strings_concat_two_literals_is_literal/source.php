<?php

declare(strict_types=1);

/** @param literal-string $s */
function takes_literal(string $s): void
{
    echo $s;
}

takes_literal('foo' . 'bar');
