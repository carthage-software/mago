<?php

declare(strict_types=1);

class Holder
{
    public string $name = '';
}

$h = new Holder();
$h->name = 'mago';
if ($h->name === 'analyzer') {
    echo 'matched';
}
