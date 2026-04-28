<?php

declare(strict_types=1);

function classesInvalidCloneNull(): void
{
    $value = null;
    /**
     * @mago-expect analysis:invalid-clone
     * @mago-expect analysis:impossible-assignment
     */
    $_ = clone $value;
}
