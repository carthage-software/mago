<?php

/**
 * @throws Exception
 */
function type_is_not_narrowed_by_nested_conditional_exit(bool $flag_1, bool $flag_2): void
{
    if ($flag_1) {
        if ($flag_2) {
            throw new Exception();
        }

        echo 'hello';
    }

    if (!$flag_1) {
        echo 'hello';
    }
}
