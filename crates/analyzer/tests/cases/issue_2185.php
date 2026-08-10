<?php

declare(strict_types=1);

function term(): never
{
    exit;
}

function handle(string $method): void
{
    switch ($method) {
        case 'OPTIONS':
            term();
        case 'GET':
            $x = 1;
            echo $x;
        case 'POST':
            term();
    }
}
