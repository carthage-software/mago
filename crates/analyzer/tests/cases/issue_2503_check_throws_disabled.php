<?php

declare(strict_types=1);

function issue_2503_check_throws_disabled(): void
{
    try {
    } catch (\RuntimeException) {
    }
}

issue_2503_check_throws_disabled();
