<?php

declare(strict_types=1);

final class ClassesPrivCallSelf
{
    public function compare(self $other): bool
    {
        return $this->secret() === $other->secret();
    }

    private function secret(): int
    {
        return 1;
    }
}

(new ClassesPrivCallSelf())->compare(new ClassesPrivCallSelf());
