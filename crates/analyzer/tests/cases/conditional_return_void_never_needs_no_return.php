<?php

declare(strict_types=1);

/**
 * @return ($c is true ? void : never)
 */
function condVoidNeverAbortUnless(bool $c): void
{
    if (!$c) {
        exit;
    }
}

/**
 * @return ($c is true ? never : void)
 */
function condVoidNeverAbortIf(bool $c): void
{
    if ($c) {
        exit;
    }
}
