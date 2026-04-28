<?php

declare(strict_types=1);

final class ClassesReadonlyMethodWrite
{
    public readonly string $id;

    public function __construct()
    {
        $this->id = 'init';
    }

    public function tryWrite(): void
    {
        /** @mago-expect analysis:invalid-property-write */
        $this->id = 'changed';
    }
}
