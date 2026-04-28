<?php

declare(strict_types=1);

final class ClassesRetA
{
}

final class ClassesRetB
{
    /** @mago-expect analysis:invalid-return-statement */
    public function get(): self
    {
        return new ClassesRetA();
    }
}
