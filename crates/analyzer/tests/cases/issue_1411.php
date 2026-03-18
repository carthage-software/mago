<?php

declare(strict_types=1);

class Y1411 {
    public static function new(): static
    {
        /** @mago-expect analysis:less-specific-return-statement */
        return new self;
    }
}

final class FinalY1411 {
    public static function new(): static
    {
        return new self;
    }
}
