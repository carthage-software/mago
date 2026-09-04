<?php

declare(strict_types=1);

namespace Issue2303;

if (rand()) {
    $comments = ['asd' => 'sdfsd'];
}

$fieldName = 'asd';

$textOutput = isset($comments[$fieldName]) ? htmlspecialchars($comments[$fieldName]) : '';

if (isset($comments[$fieldName])) {
    $textOutput = htmlspecialchars($comments[$fieldName]);
}
