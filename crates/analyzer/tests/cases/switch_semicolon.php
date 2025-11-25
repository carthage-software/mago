<?php

$a = rand(0, 1) ? 'foo' : 'bar';

switch ($a) {
    case 'foo'; // @mago-expect analysis:deprecated-feature
        echo "It's foo!";
        break;
    case 'bar'; // @mago-expect analysis:deprecated-feature
        echo "It's bar!";
        break;
}
