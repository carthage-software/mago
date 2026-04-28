<?php

declare(strict_types=1);

/**
 * @phpstan-type SealedShape array{a: int, b: string, c: bool}
 */
final class HolderCA
{
    /**
     * @return key-of<SealedShape>
     */
    public function aKey(): string
    {
        return 'a';
    }
}

/** @param 'a'|'b'|'c' $k */
function takeAbcCA(string $k): string
{
    return $k;
}

$h = new HolderCA();
takeAbcCA($h->aKey());
