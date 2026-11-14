<?php

declare(strict_types=1);

class Base
{
    private int $value = 0;

    private static int $count = 0;

    public function handle(Child $other): string
    {
        $other->value = 5;
        $value = $other->value;
        $secret = $other->secret();
        $count = Child::$count + 6;
        $made = Child::make();

        return "{$value} {$secret} {$count} {$made}";
    }

    private function secret(): int
    {
        return $this->value;
    }

    private static function make(): int
    {
        return 1;
    }
}

class Child extends Base
{
    public function peek(Child $other): int
    {
        // @mago-expect analysis:non-existent-method
        // @mago-expect analysis:mixed-return-statement
        return $other->secret();
    }
}
