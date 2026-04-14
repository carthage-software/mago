<?php

function test_mixed_increment(mixed $n): void {
    // @mago-expect analysis:mixed-operand
    // @mago-expect analysis:mixed-assignment
    $n++;

    // @mago-expect analysis:mixed-operand
    // @mago-expect analysis:mixed-assignment
    ++$n;
}

function test_mixed_decrement(mixed $n): void {
    // @mago-expect analysis:mixed-operand
    // @mago-expect analysis:mixed-assignment
    $n--;

    // @mago-expect analysis:mixed-operand
    // @mago-expect analysis:mixed-assignment
    --$n;
}
