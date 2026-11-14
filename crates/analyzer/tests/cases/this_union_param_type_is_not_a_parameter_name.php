<?php

declare(strict_types=1);

class T
{
    /**
     * @param $this|null $x
     *
     * @return $this|null
     */
    public function c($x): ?self
    {
        return $x;
    }

    /**
     * @param $this | null $x
     */
    public function spaced($x): ?self
    {
        return $x;
    }
}
