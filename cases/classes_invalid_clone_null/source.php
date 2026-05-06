<?php

declare(strict_types=1);

function classesInvalidCloneNull(): void
{
    $value = null;
    /**
     */
    $_ = clone $value;
}
