<?php

declare(strict_types=1);

$content = <<<JSON
  {
      "redirectTo": "mago.carthage.software"
  }
JSON;

/** @var mixed **/
$response = \json_decode($content, associative: true);

$redirectTo = isset($response['redirectTo']) && \is_string($response['redirectTo']) ? $response['redirectTo'] : '';

echo \strtoupper($redirectTo);
