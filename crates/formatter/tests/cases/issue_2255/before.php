<?php

final class Comparator
{
    public function isSameItem(?Item $firstItem, ?Item $secondItem) : bool
    {
        if ($firstItem !== null && $secondItem !== null && $firstItem->getIdentifier() === $secondItem->getIdentifier()) {
            return true;
        }

        return false;
    }
}
