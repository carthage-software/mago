<?php

declare(strict_types=1);

/**
 * @phpstan-type Shape array{a: int, b: string, c: bool}
 */
final class ShapeHolderL
{
    /**
     * @return key-of<Shape>
     */
    public function someKey(): string
    {
        return 'a';
    }
}

$h = new ShapeHolderL();
echo $h->someKey();
