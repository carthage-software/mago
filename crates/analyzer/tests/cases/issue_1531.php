<?php

declare(strict_types=1);

/** @return array{pai_status: string, pai_message: string} */
function pai_action_wrapper(string $_mod, string $_basefn): array
{
    return ['pai_status' => 'ok', 'pai_message' => ''];
}

function show_error_page(string $_dummy): never
{
    exit('s');
}

function test(string $mod, string $basefn): string
{
    $st = pai_action_wrapper($mod, $basefn);

    if ('none' !== $st['pai_status']) {
        switch ($st['pai_status']) {
            case 'error':
                show_error_page($st['pai_message']);
            case 'warning':
            case 'ok':
                return 'ok or warning';
        }
    }

    return '';
}

echo test('', '');
