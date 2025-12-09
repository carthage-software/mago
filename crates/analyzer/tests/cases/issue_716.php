<?php

/** @var mixed $response */
$response = json_decode('{}', true);

if (!isset($response['content'][0]['input'])) {
    throw new RuntimeException('no input');
}

if (!isset($response['content'][0]['type']) || $response['content'][0]['type'] !== 'tool_use') {
    throw new RuntimeException('wrong type');
}

if (!isset($response['content'][0]['name']) || $response['content'][0]['name'] !== 'generate_dashboard_config') {
    throw new RuntimeException('wrong name');
}
