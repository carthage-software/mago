<?php

if ($a) {
    echo 'a';
}
else {
    echo 'b';
}

if ($a) {
    echo 'a';
}
elseif ($b) {
    echo 'b';
}
else {
    echo 'c';
}

try {
    echo 'try';
}
catch (Exception $e) {
    echo 'catch';
}
finally {
    echo 'finally';
}

do {
    echo 'do';
} while ($a);
