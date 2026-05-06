<?php

/**
 */
function testPossiblyNullOperand(?int $a): int
{
    return $a + 1;
}
