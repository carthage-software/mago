<?php

function test_mixed_increment(mixed $n): void
{
    $n++;

    ++$n;
}

function test_mixed_decrement(mixed $n): void
{
    $n--;

    --$n;
}
