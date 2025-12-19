<?php

enum Status
{
    case Active;
    case Inactive;
}

/**
 * @mago-expect analysis:enum-instantiation
 * @mago-expect analysis:impossible-assignment
 */
function testEnumInstantiation(): void
{
    $_ = new Status();
}
