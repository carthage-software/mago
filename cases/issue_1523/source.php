<?php

declare(strict_types=1);

$image_info = null;
$ok = getimagesize('something.gif', $image_info);
if (!is_null($image_info)) {
    echo 'hasimageinfo';
}

$image_info2 = null;
$ok2 = getimagesizefromstring('data', $image_info2);
if (!is_null($image_info2)) {
    echo 'hasimageinfo';
}
