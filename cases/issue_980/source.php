<?php

declare(strict_types=1);

class ChatClient
{
    /**
     * @var array<int,string>
     */
    private static array $colors = ['red', 'blue', 'yellow', 'green'];

    public function __construct()
    {
        if (!next(self::$colors)) {
            reset(self::$colors);
        }
    }
}
