<?php

function responsePayload($data): void
{
    new JsonResponse(['status' => 'error', 'data' => $data], 500);

    $payload     = ['status' => 'error', 'data' => $data];
    $widePayload = [
        'status'                        => 'error',
        'much_longer_diagnostic_key'    => $data,
        'another_very_long_context_key' => $data,
    ];

    $array = [
        'a'    => 1,
        'bbbb' => 2,
        'cc'   => 3,
    ];

    $array2 = [
        'short'           => 'value',
        'much_longer_key' => 'another value',
        'key'             => 'test',
    ];
}
