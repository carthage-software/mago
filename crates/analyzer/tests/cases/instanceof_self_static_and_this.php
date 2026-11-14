<?php

declare(strict_types=1);

interface Equatable {}

class Money implements Equatable
{
    public function sameKind(Money $other): bool
    {
        if (!$other instanceof static) {
            return false;
        }

        return true;
    }

    public function ternary(Money $other): int
    {
        return $other instanceof static ? 1 : 0;
    }

    public function viaSelf(Equatable $other): static
    {
        if ($other instanceof self) {
            // @mago-expect analysis:less-specific-return-statement
            return $other;
        }

        return $this;
    }

    public function viaThis(Equatable $other): static
    {
        if ($other instanceof $this) {
            return $other;
        }

        return $this;
    }
}
