<?php

declare(strict_types=1);

class InheritedThrowFailure extends Exception
{
}

class ThrowingParent
{
    /** @throws InheritedThrowFailure */
    public function run(): void
    {
        throw new InheritedThrowFailure();
    }
}

final class SilentChild extends ThrowingParent
{
    public function run(): void
    {
    }
}
