<?php

/**
 * @param callable(): void $callback
 * @param-immediately-invoked-callable $callback
 */
function invoke_callback_from_integer_switch_case(callable $callback): void
{
    switch (2) {
        case 3:
            break;
        case 2:
            try {
                $callback();
            } catch (RuntimeException) {
            }
            break;
    }
}

function integer_switch_case_invocation_keeps_its_catch_summary(): void
{
    invoke_callback_from_integer_switch_case(static function (): void {
        throw new RuntimeException();
    });
}
