<?php

declare(strict_types=1);

/**
 * @return resource|false
 */
function proc_open_with_reused_pipes_variable()
{
    $pipes = '';

    return proc_open(['ls'], [], $pipes);
}

function sscanf_outputs_are_scalars(string $input): int|float|string|null
{
    $number = null;
    sscanf($input, '%d', $number);

    return $number;
}
