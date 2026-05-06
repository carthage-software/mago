<?php

declare(strict_types=1);

/**
 * @param array{user: array{name: string}} $data
 */
function name_of(array $data): string
{
    return $data['user']['name'];
}
