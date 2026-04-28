<?php

declare(strict_types=1);

final class ClassesArgTooSpec
{
    public function take(string $name): void
    {
        unset($name);
    }
}

function classesArgTooSpec(int $n): void
{
    /** @mago-expect analysis:invalid-argument */
    (new ClassesArgTooSpec())->take($n);
}
