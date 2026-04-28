<?php

declare(strict_types=1);

function classesAnonCapture(int $captured): int
{
    $obj = new class($captured) {
        public function __construct(public int $n)
        {
        }

        public function get(): int
        {
            return $this->n;
        }
    };

    return $obj->get();
}

echo classesAnonCapture(3);
