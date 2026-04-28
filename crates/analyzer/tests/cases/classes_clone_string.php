<?php

declare(strict_types=1);

function classesCloneString(): void
{
    $value = 'hello';
    /**
     * @mago-expect analysis:invalid-clone
     * @mago-expect analysis:impossible-assignment
     */
    $_ = clone $value;
}
