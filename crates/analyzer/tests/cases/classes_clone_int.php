<?php

declare(strict_types=1);

function classesCloneInt(): void
{
    $value = 42;
    /**
     * @mago-expect analysis:invalid-clone
     * @mago-expect analysis:impossible-assignment
     */
    $_ = clone $value;
}
