<?php

declare(strict_types=1);

final class Service
{
    public function run(): string
    {
        return 'ok';
    }
}

/**
 * @mago-expect analysis:possible-method-access-on-null
 */
function flow_method_on_possibly_null(null|Service $s): string
{
    return $s->run();
}
