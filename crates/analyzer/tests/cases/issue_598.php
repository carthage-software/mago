<?php

declare(strict_types=1);

function a(): never
{
    die();
}

function xxx(): int
{
    try {
        return 0;
    } catch (Exception) {
        a();
    }
}
