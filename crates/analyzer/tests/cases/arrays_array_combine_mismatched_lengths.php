<?php

declare(strict_types=1);

/**
 * @return array<string, int>
 */
function build(): array
{
    return array_combine(['a', 'b'], [1, 2]);
}
