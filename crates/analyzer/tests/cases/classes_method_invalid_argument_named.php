<?php

declare(strict_types=1);

final class ClassesMethodInvNamed
{
    public function take(int $value): int
    {
        return $value;
    }
}

function classesMethodInvNamed(): void
{
    /** @mago-expect analysis:invalid-argument */
    (new ClassesMethodInvNamed())->take(value: 'string');
}
