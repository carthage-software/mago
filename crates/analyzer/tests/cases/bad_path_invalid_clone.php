<?php

/**
 * @mago-expect analysis:invalid-clone
 * @mago-expect analysis:impossible-assignment
 */
function testInvalidClone(): void
{
    $value = 42;
    $_ = clone $value;
}
