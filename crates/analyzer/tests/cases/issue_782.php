<?php

declare(strict_types=1);

$list = [new DateTime()];

// we can't be sure that the list pointer did not move
// so the result of current() can be false
$first = current($list);
if ($first !== false) {
    $first->format('d-m-Y');
}
