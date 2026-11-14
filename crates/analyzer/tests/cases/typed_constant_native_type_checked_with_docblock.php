<?php

declare(strict_types=1);

final class TypedConstantNativeChecked
{
    /** @var mixed */
    // @mago-expect analysis:invalid-constant-value
    public const int X = 'a';

    /** @var positive-int */
    public const int Y = 5;
}
