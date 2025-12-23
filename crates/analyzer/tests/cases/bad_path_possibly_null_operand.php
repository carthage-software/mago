<?php

/**
 * @mago-expect analysis:possibly-null-operand
 */
function testPossiblyNullOperand(?int $a): int
{
    return $a + 1;
}
