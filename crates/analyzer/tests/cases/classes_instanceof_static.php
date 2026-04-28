<?php

declare(strict_types=1);

class ClassesInstanceofStatic
{
    public function isSelf(object $other): bool
    {
        return $other instanceof static;
    }
}

(new ClassesInstanceofStatic())->isSelf(new ClassesInstanceofStatic());
