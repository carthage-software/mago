<?php

declare(strict_types=1);

/**
 * @param array<string, mixed> $column
 */
function falsy_value_widens_truthy_mixed(array $column): string
{
    /** @mago-expect analysis:mixed-assignment */
    $width = empty($column['width']) ? 0 : $column['width'];
    if ($width) {
        return 'width';
    }

    return 'no width';
}

function empty_array_widens_truthy_mixed(mixed $value, bool $reset): string
{
    if (!$value) {
        return 'empty';
    }

    /** @mago-expect analysis:mixed-assignment */
    $value = $reset ? [] : $value;
    if ($value) {
        return 'set';
    }

    return 'reset';
}
