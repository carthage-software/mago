<?php

declare(strict_types=1);

/**
 * @mago-expect analysis:static-outside-class-scope
 * @mago-expect analysis:never-return
 */
function classesNewStaticOutside(): object
{
    return new static();
}
