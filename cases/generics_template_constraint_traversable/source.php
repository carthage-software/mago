<?php

declare(strict_types=1);

/**
 * @template T of Traversable
 *
 * @param T $iter
 *
 * @return T
 */
function gen_iter_through(Traversable $iter): Traversable
{
    return $iter;
}

$g = new ArrayIterator([1, 2, 3]);
$g2 = gen_iter_through($g);
foreach ($g2 as $v) {
    echo $v;
}
