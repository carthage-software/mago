<?php

declare(strict_types=1);

class StaticFirst
{
    public int $value = 1;
    public static int $count = 2;

    public static function invalid(): mixed
    {
        // @mago-expect analysis:undefined-variable,mixed-property-access
        return $this->value;
    }

    public function valid(): int
    {
        return $this->value + self::$count;
    }
}

class InstanceFirst
{
    public int $value = 1;
    public static int $count = 2;

    public function valid(): int
    {
        return $this->value + self::$count;
    }

    public static function invalid(): mixed
    {
        // @mago-expect analysis:undefined-variable,mixed-property-access
        return $this->value;
    }
}
