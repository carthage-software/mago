<?php

declare(strict_types=1);

class Test
{
    public string $test = '' {
        set => strtolower($value);
    }

    public function getValue(): string
    {
        return $this->test;
    }
}
