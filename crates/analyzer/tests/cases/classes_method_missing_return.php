<?php

declare(strict_types=1);

final class ClassesMissingReturn
{
    /** @mago-expect analysis:missing-return-statement */
    public function get(): int
    {
    }
}
