<?php

declare(strict_types=1);

function gen_take_iter_default(Iterator $iter): bool
{
    return $iter->valid();
}

gen_take_iter_default(new ArrayIterator([1, 2, 3]));
