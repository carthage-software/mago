<?php

declare(strict_types=1);

function inh_static_outside(): void
{
    /** @mago-expect analysis:static-outside-class-scope */
    static::go();
}
