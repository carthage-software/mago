<?php

declare(strict_types=1);

/**
 * @template T
 * @template U of T
 * @param U $u
 */
function a($u): void {
    if ($u !== true) { echo 1; }
    if ($u !== false) { echo 2; }
    if ($u === true) { echo 3; }
}

/** @template T */
final class C {
    /**
     * @template U of T
     * @param U $u
     */
    public function m($u): void {
        if ($u !== true) { echo 4; }
    }
}

a(true);
