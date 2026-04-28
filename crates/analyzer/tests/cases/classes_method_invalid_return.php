<?php

declare(strict_types=1);

final class ClassesMethodInvalidReturn
{
    /** @mago-expect analysis:invalid-return-statement */
    public function get(): string
    {
        return 42;
    }
}
