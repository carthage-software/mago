<?php

declare(strict_types=1);

/**
 * @param array<string, list<int>> $devices
 * @return array<string, string>
 */
function foreach_key_assigned_after_value(array $devices): array
{
    $result = [];

    foreach ($devices as $device => $device) {
        $result[$device] = 'hello world';
    }

    return $result;
}
