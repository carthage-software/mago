<?php

/**
 */
function testInvalidClone(): void
{
    $value = 42;
    $_ = clone $value;
}
