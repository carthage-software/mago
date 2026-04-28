<?php

declare(strict_types=1);

final class ClassesInstanceofSelf
{
    public function isSame(object $other): bool
    {
        return $other instanceof self;
    }
}

(new ClassesInstanceofSelf())->isSame(new ClassesInstanceofSelf());
