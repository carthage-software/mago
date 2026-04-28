<?php

declare(strict_types=1);

final class ClassesTypedConstInt
{
    /** @mago-expect analysis:invalid-constant-value */
    public const int X = 'string';
}
