<?php

declare(strict_types=1);

final class ClassesPromInvalidDef
{
    public function __construct(
        /** @mago-expect analysis:invalid-parameter-default-value */
        public int $count = 'hello',
    ) {
    }
}
