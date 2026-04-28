<?php

declare(strict_types=1);

final class Counter {
    public static int $count = 0;

    public static function bump(): void {
        self::$count++;
    }
}

Counter::bump();
echo Counter::$count;
