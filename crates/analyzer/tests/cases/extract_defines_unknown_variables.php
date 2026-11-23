<?php

declare(strict_types=1);

/**
 * @param array<string, mixed> $data
 */
function extract_defines_unknown_variables(array $data): mixed
{
    /** @mago-expect analysis:undefined-variable,mixed-argument */
    echo $before;

    extract($data);

    return $title;
}
