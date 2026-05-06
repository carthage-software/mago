<?php

function use_bool(bool $val): void
{
    if ($val) {
        echo 'true';
    } else {
        echo 'false';
    }
}

$restore = mb_internal_encoding();
$bool = mb_internal_encoding($restore);
use_bool($bool);

$restore = mb_http_output();
$bool = mb_http_output($restore);
use_bool($bool);

$order = mb_detect_order();
$bool = mb_detect_order($order);
use_bool($bool);
