<?php

declare(strict_types=1);

/**
 * @param list<int> $outer
 * @param list<int> $inner
 */
function foreach_value(array $outer, array $inner): void
{
    foreach ($outer as $_) {
        if (rand(0, 1) === 1) {
            echo $value; // @mago-expect analysis:possibly-undefined-variable
        }

        foreach ($inner as $value) {
        }
    }
}

/**
 * @param list<int> $outer
 */
function foreach_key(array $outer): void
{
    foreach ($outer as $_) {
        if (rand(0, 1) === 1) {
            echo $key; // @mago-expect analysis:possibly-undefined-variable
        }

        foreach (['a' => 1] as $key => $_value) {
        }
    }
}

/**
 * @param list<int> $outer
 */
function static_declaration(array $outer): void
{
    foreach ($outer as $_) {
        if (rand(0, 1) === 1) {
            echo $count; // @mago-expect analysis:possibly-undefined-variable
        }

        static $count = 1;
    }
}

/**
 * @param list<int> $outer
 */
function nested_destructuring(array $outer): void
{
    foreach ($outer as $_) {
        if (rand(0, 1) === 1) {
            echo $number; // @mago-expect analysis:possibly-undefined-variable
        }

        [[$number]] = [[1]];
    }
}
