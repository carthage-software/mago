<?php

declare(strict_types=1);

use Mago\Sdk\Extension;
use Mago\Sdk\Worker;
use Mago\Tests\Sdk\Fixture\FailingRegistrationPlugin;
use Mago\Tests\Sdk\Fixture\NoInterfaceRule;
use Mago\Tests\Sdk\Fixture\PreferArrayAnyRule;

require_once __DIR__ . '/../../../vendor/autoload.php';
require_once __DIR__ . '/Fixture/PreferArrayAnyRule.php';
require_once __DIR__ . '/Fixture/NoInterfaceRule.php';
require_once __DIR__ . '/Fixture/FailingRegistrationPlugin.php';

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
    // Linting must work although this plugin's registration fails, because the
    // worker registers analyzer plugins only for analyzer requests.
    new Extension(
        identifier: 'mago/sdk-failing-registration-test',
        name: 'Mago SDK Failing Registration Test',
        version: '0.0.0',
        analyzerPlugins: [new FailingRegistrationPlugin()],
    ),
);
$worker->run();
