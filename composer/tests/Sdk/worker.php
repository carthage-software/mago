<?php

declare(strict_types=1);

use Mago\Sdk\Extension;
use Mago\Sdk\Worker;
use Mago\Tests\Sdk\Fixture\NoInterfaceRule;
use Mago\Tests\Sdk\Fixture\PreferArrayAnyRule;

require_once __DIR__ . '/../../../vendor/autoload.php';
require_once __DIR__ . '/Fixture/PreferArrayAnyRule.php';
require_once __DIR__ . '/Fixture/NoInterfaceRule.php';

$worker = new Worker(
    new Extension(
        identifier: 'mago/sdk-iter-test',
        name: 'Mago SDK Iter Test',
        version: '0.0.0',
        linterRules: [new PreferArrayAnyRule()],
    ),
    new Extension(
        identifier: 'mago/sdk-interface-test',
        name: 'Mago SDK Interface Test',
        version: '0.0.0',
        linterRules: [new NoInterfaceRule()],
    ),
);
$worker->run();
