<?php

declare(strict_types=1);

final class ClassesInvalidPropDefault
{
    /** @mago-expect analysis:invalid-property-default-value */
    public int $count = 'string';
}
