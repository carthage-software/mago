<?php

declare(strict_types=1);

final class ClassesAssignToThis
{
    public function bad(): void
    {
        $this = null;
    }
}
