<?php

switch ($value) {
    // First label
    case 1:
        run();
        break; // End of statement

        // Next label

    case 2:
        // Fallthrough label
    case 3:
        break;
        /* Default label */
    default:
        finish();
}

switch ($value):
    // First alternative label
    case 1:
        break;

        // Next alternative label

    default:
        finish();
endswitch;
