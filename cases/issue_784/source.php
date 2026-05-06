<?php

declare(strict_types=1);

$dir = new DirectoryIterator(__DIR__);
foreach ($dir as $file) {
    if ($file->isDot()) {
        continue;
    }

    echo $file->getFilename() . "\n";
}
