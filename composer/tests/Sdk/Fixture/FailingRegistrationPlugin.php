<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Fixture;

use LogicException;
use Mago\Sdk\Analyzer\Plugin;
use Mago\Sdk\Analyzer\PluginDefinition;
use Mago\Sdk\Analyzer\PluginRegistry;

/**
 * An analyzer plugin whose registration fails. A worker runs `register()` only
 * for its first analyzer request, so a worker that serves linter requests
 * starts and lints with this plugin in place.
 */
final class FailingRegistrationPlugin implements Plugin
{
    public function getDefinition(): PluginDefinition
    {
        return new PluginDefinition(
            'mago-sdk-test-failing-registration',
            'Failing registration',
            'Fails to register, for the SDK integration test.',
        );
    }

    public function register(PluginRegistry $registry): void
    {
        throw new LogicException('A worker that serves only linter requests must not register analyzer plugins.');
    }
}
