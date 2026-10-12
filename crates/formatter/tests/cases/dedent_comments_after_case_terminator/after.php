<?php

switch ($value) {
    // First label
    case 1:
        run();
        break; // End of statement

    // After break

    case 2:
        // Empty case
    case 3:
        run();

        // After code that falls through
    case 4:
        if ($value) {
            break;
        }
        // After an if that breaks
    case 5:
        return 5;
    /* After return */
    case 6:
        throw new Exception();

    // After throw
    // Second line
    case 7:
        exit(1);
    // After exit
    case 8:
        die();
    // After die
    default:
        finish();
        break;
    // After the last break
}

foreach ($values as $value) {
    switch ($value):
        case 1:
            continue 2;

        // After continue
        case 2:
            run();
            // After code that falls through
        default:
            finish();
    endswitch;
}
