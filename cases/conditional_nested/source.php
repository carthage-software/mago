<?php

/**
 * @return 1|2|3
 */
function conditional_nested(bool $a, bool $b): int
{
    return $a ? ($b ? 1 : 2) : 3;
}
