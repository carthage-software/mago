<?php

$s = '0123456789ABCDEF';
for ($b = 16; $b < 128; $b++) {
    $s[($b >> 4) & 15]; // The only valid index type for `string('0123456789ABCDEF')` is `int<0, 15>`.
}
