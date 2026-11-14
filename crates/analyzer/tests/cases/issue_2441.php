<?php

declare(strict_types=1);

/**
 * @param array<string> $_
 */
function issue_2441_takes_string_array(array $_): void {}

function issue_2441_takes_string(string $_): void {}

function issue_2441_pattern_order(string $subject): void
{
    $slots = null;
    preg_match_all('/(?<slot>\{\{(?<slotname>\w+)\}\})/', subject: $subject, matches: $slots);

    /** @mago-expect analysis:possibly-undefined-string-array-index */
    issue_2441_takes_string_array($slots['slot']);
    /** @mago-expect analysis:possibly-undefined-string-array-index */
    issue_2441_takes_string_array($slots['slotname']);
    /** @mago-expect analysis:possibly-undefined-int-array-index */
    issue_2441_takes_string_array($slots[0]);

    issue_2441_takes_string_array($slots['slot'] ?? []);
}

function issue_2441_set_order(string $subject): void
{
    $matches = null;
    preg_match_all('/(?<slot>\{\{(?<slotname>\w+)\}\})/', $subject, $matches, PREG_SET_ORDER);

    foreach ($matches as $match) {
        issue_2441_takes_string($match['slotname'] ?? '');
        issue_2441_takes_string($match[0] ?? '');
    }
}

function issue_2441_preg_match(string $subject): void
{
    $match = [];
    if (1 === preg_match('/(?<slot>\{\{(?<slotname>\w+)\}\})/', $subject, $match)) {
        issue_2441_takes_string($match['slotname'] ?? '');
    }
}
