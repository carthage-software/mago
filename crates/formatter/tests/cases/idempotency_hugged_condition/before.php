<?php

if ($a
    || ($b && ($c || $d))) {
    echo 1;
}

if ($a
    || $b || ($c && ($d || $e))) {
    echo 2;
}

while (!($a
    || ($b && ($c || $d)))) {
    echo 3;
}

if ($a || ($b && ($c || $d))) {
    echo 4;
}
