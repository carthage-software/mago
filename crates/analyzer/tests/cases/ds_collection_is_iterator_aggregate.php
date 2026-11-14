<?php

declare(strict_types=1);

/**
 * @param Ds\Vector<int> $vector
 *
 * @return Traversable<int, int>
 *
 * @throws Exception
 */
function iterator_of(Ds\Vector $vector): Traversable
{
    return $vector->getIterator();
}
