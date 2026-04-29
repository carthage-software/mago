<?php

declare(strict_types=1);

class Registry
{
    public static int $count = 0;
}

Registry::$count = 5;
if (Registry::$count === 6) {
    echo 'six';
}
