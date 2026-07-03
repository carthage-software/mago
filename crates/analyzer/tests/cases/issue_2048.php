<?php

declare(strict_types=1);

final class Repro
{
    /**
     * @param string[] $listA
     * @param string[] $listB
     */
    public function repro(string $needle, array $listA, array $listB): void
    {
        if (in_array($needle, $listA, strict: true)) {
            return;
        }

        if (!in_array($needle, $listB, strict: true)) {
            return;
        }

        if (in_array($needle, $listB, strict: true)) {
            return;
        }
    }
}
