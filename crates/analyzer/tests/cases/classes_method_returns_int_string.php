<?php

declare(strict_types=1);

final class ClassesIntRetStr
{
    /** @mago-expect analysis:invalid-return-statement */
    public function get(): int
    {
        return 'forty-two';
    }
}
