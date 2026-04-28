<?php

declare(strict_types=1);

function inh_self_outside(): void
{
    /** @mago-expect analysis:self-outside-class-scope */
    self::go();
}
