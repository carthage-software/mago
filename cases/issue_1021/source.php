<?php

declare(strict_types=1);

#[NoDiscard]
function foo(): string
{
    return 'test';
}

(void) foo();
