<?php

declare(strict_types=1);

final class ClassesTypedConstMismatch
{
    /** @mago-expect analysis:invalid-constant-value */
    public const string ID = 123;
}
