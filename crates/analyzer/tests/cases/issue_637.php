<?php

/**
 * @param array<int, string> $array
 * @mago-expect analysis:redundant-condition
 */
function foo(array $array): void
{
    foreach ($array as $key => $value) {
        if (is_numeric($key)) {
            echo "Key is numeric\n";
        }
    }
}

/**
 * @param array<string, string> $assoc
 * @mago-expect analysis:redundant-condition
 */
function bar(array $assoc): void
{
    foreach ($assoc as $key => $value) {
        if (is_string($key)) {
            echo "Key is string\n";
        }
    }
}

function baz(array $generic): void
{
    foreach ($generic as $key => $_) {
        if (is_numeric($key)) {
            echo "Key is numeric\n";
        }
    }
}
