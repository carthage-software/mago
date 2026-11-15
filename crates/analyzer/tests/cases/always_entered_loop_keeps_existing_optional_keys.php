<?php

declare(strict_types=1);

/**
 * @param array{'a'?: int} $values
 *
 * @return array{'a': int}
 */
function untouched(array $values): array
{
    for ($i = 0; $i < 3; $i++) {
    }

    // @mago-expect analysis:invalid-return-statement
    return $values;
}

/**
 * @param array{'a'?: int} $values
 *
 * @return array{'a'?: int, 'b': int}
 */
function adds_a_key(array $values): array
{
    for ($i = 0; $i < 3; $i++) {
        $values['b'] = $i;
    }

    return $values;
}

/**
 * @param array{'a': int} $values
 *
 * @return array{'a': int}
 */
function may_unset_a_key(array $values, bool $flag): array
{
    $i = 0;
    while ($i < 3) {
        if ($flag) {
            unset($values['a']);
        }

        $i++;
    }

    // @mago-expect analysis:invalid-return-statement
    return $values;
}

function writes_a_new_array(): int
{
    $work = [];
    for ($i = 0; $i < 3; $i++) {
        $work[$i] = $i;
    }

    return $work[0];
}
