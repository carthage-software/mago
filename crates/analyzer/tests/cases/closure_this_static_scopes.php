<?php

declare(strict_types=1);

class Widget
{
    public function run(): void
    {
        // @mago-expect analysis:undefined-variable
        $arrow = static fn () => $this;
        // @mago-expect analysis:undefined-variable
        $closure = static function () {
            return $this;
        };
        // @mago-expect analysis:undefined-variable
        $nestedArrow = static fn () => fn () => $this;
        // @mago-expect analysis:undefined-variable
        $nestedClosure = static function () {
            return function () {
                return $this;
            };
        };
    }

    public static function withoutInstance(): void
    {
        // @mago-expect analysis:undefined-variable
        $arrow = fn () => $this;
        // @mago-expect analysis:undefined-variable
        $closure = function () {
            return $this;
        };
    }
}
