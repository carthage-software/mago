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
 */
function flow_method_on_possibly_null(?Service $s): string
{
    return $s->run();
}
