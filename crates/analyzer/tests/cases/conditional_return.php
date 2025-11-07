<?php

/**
 * Dumps information about a variable for debugging purposes.
 *
 * @return ($return is true ? string : null)
 */
function debug(mixed $value, bool $return = false): null|string
{
    if ($return) {
        return var_export($value, true);
    }

    echo '--- debug --- ';
    echo '[' . gettype($value) . '] ';
    echo var_export($value, true) . "\n";
    echo '-------------' . "\n";

    return null;
}

debug('Hello, World!'); // ok
$result = debug(42, true); // ok
echo $result; // ok
