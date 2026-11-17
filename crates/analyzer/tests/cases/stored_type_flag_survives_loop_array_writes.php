<?php

declare(strict_types=1);

/**
 * @param array<array-key, mixed>|stdClass $record
 *
 * @return array<array-key, mixed>|stdClass
 */
function sanitize_record(array|stdClass $record): array|stdClass
{
    $fields = ['id', 'url', 'name'];

    if (is_object($record)) {
        $is_object = true;
    } else {
        $is_object = false;
    }

    foreach ($fields as $field) {
        if ($is_object) {
            if (isset($record->$field)) {
                $record->$field = trim((string) $record->$field);
            }
        } else {
            if (isset($record[$field])) {
                $record[$field] = trim((string) $record[$field]);
            }
        }
    }

    return $record;
}
