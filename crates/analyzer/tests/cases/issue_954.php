<?php

declare(strict_types=1);

// $_FILES should be potentially empty, not non-empty-array
if (count($_FILES) != 0) {
    echo "not empty!";
}
