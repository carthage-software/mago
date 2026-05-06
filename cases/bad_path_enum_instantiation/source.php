<?php

enum Status
{
    case Active;
    case Inactive;
}

/**
 */
function testEnumInstantiation(): void
{
    $_ = new Status();
}
