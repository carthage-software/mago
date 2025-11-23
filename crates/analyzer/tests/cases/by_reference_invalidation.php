<?php

function postSort(): void
{
    $a = ['b', 'a'];

    if ($a[0] === 'a') { // @mago-expect analysis:redundant-comparison,impossible-condition
    }

    sort($a);

    if ($a[0] === 'a') {
        echo 'good';
    }

    if ($a[0] === 'b') {
        echo 'also good';
    }
}

function multipleSorts(): void
{
    $arr = [1, 2, 3];

    if ($arr[0] === 2) { // @mago-expect analysis:redundant-comparison,impossible-condition
    }

    if ($arr[1] === 3) { // @mago-expect analysis:redundant-comparison,impossible-condition
    }

    if ($arr[2] === 1) { // @mago-expect analysis:redundant-comparison,impossible-condition
    }

    shuffle($arr);

    // After shuffle: order is unknown - no errors
    if ($arr[0] === 1) {
        echo 'could be';
    }

    if ($arr[1] === 2) {
        echo 'could be';
    }

    if ($arr[2] === 3) {
        echo 'could be';
    }
}
