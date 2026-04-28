<?php

declare(strict_types=1);

final class ClassesPromVisCombo
{
    public function __construct(
        public int $a,
        protected int $b,
        private int $c,
        public readonly string $d,
    ) {
    }

    public function sum(): int
    {
        return $this->a + $this->b + $this->c;
    }

    public function id(): string
    {
        return $this->d;
    }
}

$obj = new ClassesPromVisCombo(1, 2, 3, 'x');
echo $obj->sum();
echo $obj->id();
