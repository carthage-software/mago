<?php

declare(strict_types=1);

abstract class Base
{
    public function __toString(): string
    {
        return '';
    }

    public function add(string|Stringable|null $arg): static
    {
        return $this;
    }
}

final class Andx extends Base {}

function useImplicit(Base $or, Andx $and): void
{
    $or->add($and);
    useExplicit($or, $and);
}

function useExplicit(Base $or, Stringable $s): void
{
    $or->add($s);
}
