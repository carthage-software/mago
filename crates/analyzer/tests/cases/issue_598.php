<?php

declare(strict_types=1);

function a(): never
{
    die();
}

/** @mago-expect analysis:dead-catch */
function xxx(): int
{
    try {
        return 0;
    } catch (Exception) {
        a();
    }
}
