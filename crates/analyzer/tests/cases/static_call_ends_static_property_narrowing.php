<?php

declare(strict_types=1);

final class Registry
{
    public static ?Registry $instance = null;

    public static function reset(): void
    {
        self::$instance = null;
    }

    public static function current(): Registry
    {
        if (self::$instance === null) {
            exit(1);
        }

        self::reset();

        // @mago-expect analysis:nullable-return-statement,invalid-return-statement
        return self::$instance;
    }
}

final class Single
{
    private static ?Single $instance = null;

    public function init(): void {}

    public static function get(): Single
    {
        if (self::$instance === null) {
            self::$instance = new self();
        }

        self::$instance->init();

        return self::$instance;
    }
}
