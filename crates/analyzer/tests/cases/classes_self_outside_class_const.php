<?php

declare(strict_types=1);

/**
 * @mago-expect analysis:self-outside-class-scope
 * @mago-expect analysis:never-return
 */
function classesSelfConstOutside(): string
{
    return self::class;
}
