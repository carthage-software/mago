<?php

declare(strict_types=1);

function classesInvalidCloneBool(): void
{
    $value = true;
    /**
     * @mago-expect analysis:invalid-clone
     * @mago-expect analysis:impossible-assignment
     */
    $_ = clone $value;
}
