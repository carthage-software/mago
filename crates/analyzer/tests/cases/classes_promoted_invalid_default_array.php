<?php

declare(strict_types=1);

final class ClassesPromInvDefArr
{
    public function __construct(
        /** @mago-expect analysis:invalid-parameter-default-value */
        public string $name = [],
    ) {
    }
}
