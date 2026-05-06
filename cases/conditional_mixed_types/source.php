<?php

/**
 * @return 1|'hello'
 */
function conditional_mixed_types(bool $a): int|string
{
    return $a ? 1 : 'hello';
}
