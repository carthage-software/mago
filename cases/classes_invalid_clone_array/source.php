<?php

declare(strict_types=1);

function classesInvalidCloneArr(): void
{
    $value = [1, 2, 3];
    /**
     */
    $_ = clone $value;
}
