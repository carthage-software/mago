<?php

declare(strict_types=1);

final class ClassesMethodChain
{
    public function a(): self
    {
        return $this;
    }

    public function b(): self
    {
        return $this;
    }
}

(new ClassesMethodChain())->a()->b();
