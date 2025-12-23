<?php

/**
 * @mago-expect analysis:null-operand
 * @mago-expect analysis:mixed-return-statement
 */
function testNullOperand(): int
{
    $a = null;
    return $a + 1;
}
