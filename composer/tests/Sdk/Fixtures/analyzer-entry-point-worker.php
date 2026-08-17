<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Fixtures;

use Mago\Sdk\Analyzer\MethodTarget;
use Mago\Sdk\Analyzer\Plugin;
use Mago\Sdk\Analyzer\PluginDefinition;
use Mago\Sdk\Analyzer\PluginRegistry;
use Mago\Sdk\Extension;
use Mago\Sdk\Worker;

use function dirname;

require_once dirname(__DIR__, 4) . '/vendor/autoload.php';

/**
 * @mago-expect lint:file-name
 */
final class EntryPointPlugin implements Plugin
{
    public function getDefinition(): PluginDefinition
    {
        return new PluginDefinition(
            identifier: 'entry-point-proof',
            name: 'Entry point proof',
            description: 'Exercises native declarative framework entry-point resolution.',
        );
    }

    public function register(PluginRegistry $registry): void
    {
        $registry->registerEntryPoint(MethodTarget::exact('FrameworkTestCase', 'action*'));
        $registry->registerEntryPoint(MethodTarget::exact('FrameworkTestCase', 'inheritedEntry'));
        $registry->registerEntryPoint(MethodTarget::exact('FrameworkTestCase', 'traitEntry'));
        $registry->registerAttributedEntryPoint('FrameworkTestCase', 'FrameworkEntry');
    }
}

(new Worker(new Extension(
    identifier: 'mago/entry-point-proof',
    name: 'Mago entry-point extension fixture',
    version: '1.0.0',
    analyzerPlugins: [new EntryPointPlugin()],
)))->run();
