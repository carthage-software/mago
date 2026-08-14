<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Unit;

use Mago\Sdk\Analyzer\Plugin;
use Mago\Sdk\Analyzer\PluginDefinition;
use Mago\Sdk\Analyzer\PluginRegistry;
use Mago\Sdk\Exception\InvalidArgumentException;
use Mago\Sdk\Extension;
use Mago\Sdk\Worker;
use PHPUnit\Framework\TestCase;

final class WorkerTest extends TestCase
{
    public function testExtensionIdentifiersAreCaseInsensitive(): void
    {
        $this->expectException(InvalidArgumentException::class);
        $this->expectExceptionMessage('registered more than once');

        new Worker(new Extension('Acme/Tools', 'Acme', '1.0.0'), new Extension('acme/tools', 'Acme', '2.0.0'));
    }

    public function testPluginAliasesAreUniqueAcrossExtensions(): void
    {
        $this->expectException(InvalidArgumentException::class);
        $this->expectExceptionMessage('selector `SHARED` is shared by plugins `first` and `second`');

        new Worker(new Extension('acme/first', 'First', '1.0.0', analyzerPlugins: [
            self::plugin(new PluginDefinition('first', 'First', 'First plugin.', ['shared'])),
        ]), new Extension('acme/second', 'Second', '1.0.0', analyzerPlugins: [
            self::plugin(new PluginDefinition('second', 'Second', 'Second plugin.', ['SHARED'])),
        ]));
    }

    private static function plugin(PluginDefinition $definition): Plugin
    {
        return new class($definition) implements Plugin {
            public function __construct(
                private readonly PluginDefinition $definition,
            ) {}

            public function getDefinition(): PluginDefinition
            {
                return $this->definition;
            }

            public function register(PluginRegistry $registry): void {}
        };
    }
}
