<?php

declare(strict_types=1);

final class Service
{
    public function run(): string
    {
        return 'ok';
    }
}

function flow_assert_instanceof(object $o): string
{
    assert($o instanceof Service);

    return $o->run();
}
