<?php

/**
 * @return 0|1|3|5
 */
function test_switch(string $value): int
{
    $result = 0;
    switch ($value) {
        case '1':
            $result = 1;
            break;
        case '2':
            $result = 2;
        case '3':
            $result += 3;
            break;
    }

    return $result;
}
