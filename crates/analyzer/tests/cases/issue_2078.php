<?php

declare(strict_types=1);

function test(): int
{
    $violations = 0;
    $autoescape = false;
    $state = 0;

    for ($i = 0; $i < 99; $i++) {
        switch ($state) {
            case 0:
                if ($autoescape) {
                    ++$violations;
                }
                $state = 1;
                break;

            case 1:
                $autoescape = !$autoescape;
                $state = 0;
                break;
        }
    }

    return $violations;
}
