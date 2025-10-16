<?php

declare(strict_types=1);

/**
 * @psalm-inheritors Left|Right
 */
interface Either
{
    public function isLeft(): bool;
}

interface Left extends Either
{
    public function getLeft(): int;
}

interface Right extends Either
{
    public function getRight(): string;
}

function process(Either $either): string
{
    if ($either instanceof Left) {
        return (string) $either->getLeft();
    } else {
        return $either->getRight();
    }
}
