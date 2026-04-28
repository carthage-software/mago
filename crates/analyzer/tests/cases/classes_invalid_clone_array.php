<?php

declare(strict_types=1);

function classesInvalidCloneArr(): void
{
    $value = [1, 2, 3];
    /**
     * @mago-expect analysis:invalid-clone
     * @mago-expect analysis:impossible-assignment
     */
    $_ = clone $value;
}
