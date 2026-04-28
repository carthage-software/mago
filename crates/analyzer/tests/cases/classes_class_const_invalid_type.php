<?php

declare(strict_types=1);

final class ClassesConstInvType
{
    /** @mago-expect analysis:invalid-constant-value */
    public const string FOO = 0;
}
