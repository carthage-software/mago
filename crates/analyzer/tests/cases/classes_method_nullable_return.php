<?php

declare(strict_types=1);

final class ClassesMethodNullableRet
{
    /**
     * @mago-expect analysis:nullable-return-statement
     * @mago-expect analysis:invalid-return-statement
     */
    public function get(null|string $value): string
    {
        return $value;
    }
}
