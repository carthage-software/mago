<?php

declare(strict_types=1);

/** @param false|string[] $row */
function test_empty_narrows_false(false|array $row): int
{
    if (!empty($row['object_id'])) {
        return (int) $row['object_id'];
    }

    return 0;
}
