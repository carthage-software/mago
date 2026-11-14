<?php

declare(strict_types=1);

function guarded(): void
{
    if (function_exists('my_optional_function')) {
        $arrow = static fn(): mixed => my_optional_function();
        $closure = static function (): mixed {
            return my_optional_function();
        };

        var_dump($arrow(), $closure());
    }

    if (defined('MY_OPTIONAL_FLAG')) {
        $flag = static fn(): mixed => MY_OPTIONAL_FLAG;

        var_dump($flag());
    }
}
