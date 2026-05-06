<?php

declare(strict_types=1);

final class Foo
{
    public int $value = 0;

    public function update(): void
    {
        $this->value = 'string';
    }
}

new Foo();
