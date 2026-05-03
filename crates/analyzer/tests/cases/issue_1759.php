<?php

declare(strict_types=1);

function read_after_conditional_assignment(int $show_details): void
{
    if ($show_details) {
        $details = 'foo';
    }

    if ($show_details) {
        /** @mago-expect analysis:possibly-undefined-variable */
        var_dump($details);
    }
}

function isset_suppresses_after_conditional_assignment(int $show_details): void
{
    if ($show_details) {
        $value = 42;
    }

    if (isset($value)) {
        var_dump($value);
    }
}
