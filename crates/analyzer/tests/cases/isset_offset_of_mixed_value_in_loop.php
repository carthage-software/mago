<?php

declare(strict_types=1);

/**
 * @param list<mixed> $rows
 */
function isset_offset_of_mixed_value_in_loop(array $rows, string $field): int
{
    $found = 0;
    /** @mago-expect analysis:mixed-assignment */
    foreach ($rows as $row) {
        /** @mago-expect analysis:mixed-assignment */
        $id = isset($row[$field]) ? $row[$field] : false;
        if ($id) {
            $found++;
        }
    }

    return $found;
}
