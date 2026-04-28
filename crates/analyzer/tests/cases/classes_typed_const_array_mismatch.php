<?php

declare(strict_types=1);

final class ClassesTypedConstArr
{
    /** @mago-expect analysis:invalid-constant-value */
    public const array DATA = 'not-array';
}
