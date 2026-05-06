<?php

$a = rand(0, 1) ? 'foo' : 'bar';

switch ($a) {
    case 'foo';
        echo "It's foo!";
        break;
    case 'bar';
        echo "It's bar!";
        break;
}
