<?php

declare(strict_types=1);

function classesInvalidCloneBool(): void
{
    $value = true;
    /**
     */
    $_ = clone $value;
}
