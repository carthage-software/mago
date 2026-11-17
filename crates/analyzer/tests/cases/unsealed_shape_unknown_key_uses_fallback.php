<?php

declare(strict_types=1);

/**
 * @param array{c: null, ...<string, int>} $values
 */
function unsealed_shape_merge(array $values, bool $add): int
{
    if ($add) {
        $values['k'] = 'str';
    }

    /** @mago-expect analysis:possibly-undefined-string-array-index */
    return $values['other'];
}

/**
 * @param array{c: null, ...<string, int>} $values
 */
function unsealed_shape_narrowing(array $values): int
{
    /** @mago-expect analysis:possibly-undefined-string-array-index */
    if ($values['k'] === 5) {
        echo 'five';
    }

    return $values['k'];
}

/**
 * @param array{c: list<string>, ...<string, int>} $values
 */
function unsealed_shape_positive(array $values): int
{
    /** @mago-expect analysis:possibly-undefined-string-array-index */
    if ($values['j'] > 0) {
        return $values['j'];
    }

    return 0;
}
