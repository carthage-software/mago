<?php

declare(strict_types=1);

final class ClassesStaticInvDef
{
    /** @mago-expect analysis:invalid-property-default-value */
    public static int $count = 'string';
}
