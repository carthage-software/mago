<?php

$cond = mt_rand();
$val = '';
switch ($cond) {
    case 0:
        $val = 'first';
    // fall-through
    case 1:
        if (!$val) {
            $second = 'second';
        }
}
