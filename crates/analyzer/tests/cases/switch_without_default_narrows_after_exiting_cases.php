<?php

declare(strict_types=1);

/**
 * @param 'a'|'b'|'c' $letter
 *
 * @return 'c'
 */
function switch_exiting_cases_narrow_subject(string $letter): string
{
    switch ($letter) {
        case 'a':
            return 'c';
        case 'b':
            return 'c';
    }

    return $letter;
}

/**
 * @param 'a'|'b'|'c' $letter
 */
function switch_breaking_case_keeps_subject(string $letter): string
{
    switch ($letter) {
        case 'a':
            break;
        case 'b':
            return 'b';
    }

    return $letter;
}
