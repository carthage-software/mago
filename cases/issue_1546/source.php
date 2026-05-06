<?php

declare(strict_types=1);

$str = parse_url(':', PHP_URL_SCHEME);

if (false === $str) {
    echo 'false';
}

$port = parse_url('http://example.com:8080', PHP_URL_PORT);

if (false === $port) {
    echo 'false';
}
