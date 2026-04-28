<?php

declare(strict_types=1);

final class Service
{
    public function run(): string
    {
        return 'ok';
    }
}

function flow_method_on_null_narrow(): string
{
    $s = null;
    /** @mago-expect analysis:method-access-on-null,mixed-return-statement */
    return $s->run();
}
