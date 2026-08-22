<?php

declare(strict_types=1);

namespace A;

function issue_2233_switch(object $foo): void
{
    while (true) {
        switch ($foo) {
            default:
                continue 2;
        }
    }
}

function issue_2233_nested_loop(object $foo): void
{
    while (true) {
        while (true) {
            switch ($foo) {
                default:
                    continue 3;
            }
        }
    }
}

function issue_2233_nested_switch(object $foo): void
{
    while (true) {
        switch ($foo) {
            default:
                switch ($foo) {
                    default:
                        continue 3;
                }
        }
    }
}

function issue_2233_invalid_level(object $foo): void
{
    while (true) {
        switch ($foo) {
            default:
                // @mago-expect analysis:invalid-continue
                continue 3;
        }
    }
}

function issue_2233_zero_level(): void
{
    while (true) {
        // @mago-expect analysis:invalid-continue
        continue 0;
    }
}

function issue_2233_continue_to_switch(object $foo): mixed
{
    $result = 'initial';

    switch ($foo) {
        default:
            while (true) {
                $result = 1;
                continue 2;
            }
    }

    return $result;
}

function issue_2233_continue_between_switches(object $foo): void
{
    switch ($foo) {
        default:
            switch ($foo) {
                default:
                    continue 2;
            }
    }
}

function issue_2233_continue_from_switch(object $foo): void
{
    switch ($foo) {
        default:
            continue;
    }
}

function issue_2233_break_switch(object $foo): void
{
    while (true) {
        switch ($foo) {
            default:
                break 2;
        }
    }
}

function issue_2233_break_nested_loop(object $foo): void
{
    while (true) {
        while (true) {
            switch ($foo) {
                default:
                    break 3;
            }
        }
    }
}

function issue_2233_break_nested_switch(object $foo): void
{
    while (true) {
        switch ($foo) {
            default:
                switch ($foo) {
                    default:
                        break 3;
                }
        }
    }
}

function issue_2233_break_to_switch(object $foo): mixed
{
    $result = 'initial';

    switch ($foo) {
        default:
            while (true) {
                $result = 1;
                break 2;
            }
    }

    return $result;
}

function issue_2233_invalid_break_level(object $foo): void
{
    while (true) {
        switch ($foo) {
            default:
                // @mago-expect analysis:invalid-break
                break 3;
        }
    }
}

function issue_2233_zero_break_level(): void
{
    while (true) {
        // @mago-expect analysis:invalid-break
        break 0;
    }
}
