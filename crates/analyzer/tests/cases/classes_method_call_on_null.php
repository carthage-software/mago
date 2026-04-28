<?php

declare(strict_types=1);

final class ClassesMcallOnNull
{
    public function ping(): string
    {
        return 'pong';
    }
}

function classesMcallNull(): void
{
    $obj = null;
    /** @mago-expect analysis:method-access-on-null */
    $obj->ping();
}
