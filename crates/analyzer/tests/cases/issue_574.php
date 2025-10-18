<?php

function first_example(): void {
    $_SERVER['debug'] = (
        ($_SERVER['production'] ?? '0') !== '1'
    );

    if ($_SERVER['debug']) {
        echo "Debug mode: errors are shown.";
    } else {
        echo "Production mode: errors are hidden.";
    }
}

function second_example(): void {
    if (isset($_SERVER['argv'])) {
        foreach ($_SERVER['argv'] as $argument) {
            echo "Argument: " . $argument . "\n";
        }
    }
}

// @mago-expect analysis:redundant-logical-operation
function third_example(): void {
    // force $_SERVER to be registered in block context - works
    $_ =  $_SERVER['argv'] ?? [];

    if (isset($_SERVER['argv']) && is_array($_SERVER['argv'])) {
        foreach ($_SERVER['argv'] as $argument) {
            echo "Argument: " . $argument . "\n";
        }
    }
}


// @mago-expect analysis:redundant-logical-operation
function forth_example(): void {
    if (isset($_SERVER['argv']) && is_array($_SERVER['argv'])) {
        foreach ($_SERVER['argv'] as $argument) {
            echo "Argument: " . $argument . "\n";
        }
    }
}
