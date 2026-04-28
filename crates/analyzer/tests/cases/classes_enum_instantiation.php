<?php

declare(strict_types=1);

enum ClassesEnumInstantiation
{
    case A;
    case B;
}

/**
 * @mago-expect analysis:enum-instantiation
 * @mago-expect analysis:impossible-assignment
 */
$_ = new ClassesEnumInstantiation();
