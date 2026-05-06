<?php

declare(strict_types=1);

if (PHP_VERSION_ID >= 80000) {
    function conditionalFunction(): int
    {
        return 1;
    }
} else {
    function conditionalFunction(): int
    {
        return 2;
    }
}

if (PHP_VERSION_ID >= 80000) {
    class Conditional {}
} else {
    class Conditional {}
}
