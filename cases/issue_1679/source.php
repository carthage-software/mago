<?php

declare(strict_types=1);

$t = ['some' => 'thing'];
if (!empty($t['info'])) {
    echo "got some info\n";
}

$t = ['some' => 'other'];
if (isset($t['info'])) {
    echo "got some info set\n";
}

$t = ['some' => 'third'];
if (array_key_exists('info', $t)) {
    echo "got some info set\n";
}
