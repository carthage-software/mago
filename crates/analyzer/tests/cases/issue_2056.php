<?php

declare(strict_types=1);

namespace Webware\CommandBus;

interface CommandBusInterface {}

function consume(array $config): void
{
    /**
     * @var array{
     *   dependencies: array<string, mixed>,
     *   Webware\CommandBus\CommandBusInterface: array{
     *     command_map: array<class-string, class-string>,
     *     middleware_pipeline: array<array{middleware: class-string, priority?: int}>,
     *   },
     * } $config
     */

    $busConfig = $config[CommandBusInterface::class];
    echo count($busConfig['middleware_pipeline']);
}
