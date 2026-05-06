<?php

declare(strict_types=1);

final class ClassesPromInvalidDef
{
    public function __construct(
        public int $count = 'hello',
    ) {}
}
