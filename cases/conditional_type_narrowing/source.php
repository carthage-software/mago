<?php

function conditional_type_narrowing(int|string $a): int
{
    return is_int($a) ? $a + 1 : strlen($a);
}
