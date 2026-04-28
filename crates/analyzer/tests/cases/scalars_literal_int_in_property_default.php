<?php

declare(strict_types=1);

final class Counter {
    public int $value = 0;

    public function increment(): void {
        $this->value++;
    }
}
