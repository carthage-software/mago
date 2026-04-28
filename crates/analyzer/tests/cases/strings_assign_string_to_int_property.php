<?php

declare(strict_types=1);

final class Foo
{
    public int $value = 0;

    public function update(): void
    {
        /** @mago-expect analysis:invalid-property-assignment-value */
        $this->value = 'string';
    }
}

new Foo();
