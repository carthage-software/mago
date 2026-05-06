<?php

declare(strict_types=1);

class X1410
{
    /** @return $this */
    public function chained(): self
    {
        return new self();
    }
}

final class FinalX1410
{
    /** @return $this */
    public function chained(): self
    {
        return new self();
    }
}
