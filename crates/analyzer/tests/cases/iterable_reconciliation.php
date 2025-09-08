<?php

/**
 * @return array{mixed, mixed}
 *
 * @mago-expect analysis:mixed-assignment
 */
function get_first_pair(mixed $mixed): array
{
    if (is_iterable($mixed)) {
        foreach ($mixed as $k => $v) {
            return [$k, $v];
        }
    }

    return [null, null];
}
