<?php

declare(strict_types=1);

function classesInvalidCloneFloat(): void
{
    $value = 3.14;
    /**
     * @mago-expect analysis:invalid-clone
     * @mago-expect analysis:impossible-assignment
     */
    $_ = clone $value;
}
