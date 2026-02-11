<?php

declare(strict_types=1);

/**
 * @param array<string, string> $team
 */
function foo(array $team): string
{
    $teamType = strtolower($team['teamType'] ?? '');

    $label = $teamType !== '' ? strtoupper($teamType) : 'UNKNOWN';
    $backgroundColor = $teamType === 'permanent' ? '#1e64ff' : '#6c757d';
    $borderColor = $teamType === 'permanent' ? '#1346b0' : '#4f5a63';

    return (
        '<div style="display: inline-block; font-size: 10px; padding: 2px 6px; border-radius: 3px; '
        . 'background: '
        . $backgroundColor
        . '; color: #ffffff; border: 1px solid '
        . $borderColor
        . ';">'
        . $label
        . '</div>'
    );
}
