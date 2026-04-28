<?php

declare(strict_types=1);

final class ClassesMethodFalsableRet
{
    /**
     * @mago-expect analysis:falsable-return-statement
     * @mago-expect analysis:invalid-return-statement
     */
    public function get(false|string $value): string
    {
        return $value;
    }
}
