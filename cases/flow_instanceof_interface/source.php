<?php

declare(strict_types=1);

interface Greeter
{
    public function greet(): string;
}

final class Hello implements Greeter
{
    public function greet(): string
    {
        return 'hi';
    }
}

function flow_instanceof_interface(object $o): string
{
    if ($o instanceof Greeter) {
        return $o->greet();
    }

    return '';
}
