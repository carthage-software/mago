<?php

declare(strict_types=1);

$value = random_int(min: 1, max: 3);
const CASE_TWO = 2;

switch ($value) {
    case 1:
        echo '1';
        break;
    case 2:
        echo '2';
        break;
    // @mago-expect analysis:unreachable-switch-case
    case 2:
        echo '2.1';
        break;
    case 3:
        echo '3';
        break;
}

switch ($value) {
    case 1 + 1:
        echo '2';
        break;
    // @mago-expect analysis:unreachable-switch-case
    case 2:
        echo '2.1';
        break;
}

switch ($value) {
    case CASE_TWO:
        echo '2';
        break;
    // @mago-expect analysis:unreachable-switch-case
    case 2:
        echo '2.1';
        break;
}

switch ($value) {
    case 2:
    // @mago-expect analysis:duplicate-switch-case
    case 2:
        echo '2';
        break;
}

switch ($value) {
    case 2:
        echo '2';
    // @mago-expect analysis:duplicate-switch-case
    case 2:
        echo '2.1';
        break;
}

switch ($value) {
    case 2:
        echo '2';
        break;
    // @mago-expect analysis:unreachable-switch-case
    case 2:
        echo '2.1';
    // @mago-expect analysis:unreachable-switch-case
    case 2:
        echo '2.2';
        break;
}
