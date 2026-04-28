<?php

declare(strict_types=1);

final class ClassesAssignToThis
{
    public function bad(): void
    {
        /** @mago-expect analysis:assignment-to-this */
        $this = null;
    }
}
