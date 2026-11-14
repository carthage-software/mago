<?php

declare(strict_types=1);

/**
 * @throws Exception
 */
function count_array(iterable $input): int
{
    while ($input instanceof IteratorAggregate) {
        $input = $input->getIterator();
    }

    return is_array($input) ? count($input) : 0;
}
