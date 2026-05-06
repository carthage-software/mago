<?php

declare(strict_types=1);

function classesInvalidCloneFloat(): void
{
    $value = 3.14;
    /**
     */
    $_ = clone $value;
}
