<?php

declare(strict_types=1);

namespace Mago;

use Composer\Composer;
use Composer\DependencyResolver\Operation\InstallOperation;
use Composer\DependencyResolver\Operation\UpdateOperation;
use Composer\EventDispatcher\EventSubscriberInterface;
use Composer\Installer\PackageEvent;
use Composer\Installer\PackageEvents;
use Composer\IO\IOInterface;
use Composer\Plugin\Capability\CommandProvider;
use Composer\Plugin\Capable;
use Composer\Plugin\PluginInterface;
use Composer\Util\ProcessExecutor;
use Symfony\Component\Process\PhpExecutableFinder;

use function array_map;
use function assert;
use function getenv;
use function implode;

final class MagoPlugin implements PluginInterface, EventSubscriberInterface, Capable
{
    public const PACKAGE_NAME = 'carthage-software/mago';

    /**
     * @inheritDoc
     */
    public function activate(Composer $composer, IOInterface $io): void
    {
    }

    /**
     * @inheritDoc
     */
    public function deactivate(Composer $composer, IOInterface $io): void
    {
    }

    /**
     * @inheritDoc
     */
    public function uninstall(Composer $composer, IOInterface $io): void
    {
    }

    /**
     * @return array<class-string, class-string>
     */
    public function getCapabilities(): array
    {
        return [
            CommandProvider::class => MagoCommandProvider::class,
        ];
    }

    /**
     * Attach package installation events:.
     *
     * {@inheritdoc}
     */
    public static function getSubscribedEvents(): array
    {
        return [
            PackageEvents::POST_PACKAGE_INSTALL => 'onPackageEvent',
            PackageEvents::POST_PACKAGE_UPDATE => 'onPackageEvent',
        ];
    }

    /**
     * @mago-expect lint:strictness/no-shorthand-ternary
     */
    public function onPackageEvent(PackageEvent $event): void
    {
        if (!$this->isMagoPackageEvent($event)) {
            return;
        }

        $composer = $event->getComposer();
        $loop = $composer->getLoop();
        $command_executor = $loop->getProcessExecutor();
        assert(
            $command_executor instanceof ProcessExecutor,
            'Expecting a process executor, but none was found on the composer loop.',
        );

        $command_executor->executeTty(implode(' ', [
            (new PhpExecutableFinder())->find() ?: 'php',
            ...array_map(ProcessExecutor::escape(...), [
                getenv('COMPOSER_BINARY') ?: 'composer',
                'mago:install-binary',
            ]),
        ]));
    }

    /**
     * @mago-expect lint:best-practices/no-else-clause
     */
    private function isMagoPackageEvent(PackageEvent $event): bool
    {
        $operation = $event->getOperation();
        $package = null;
        if ($operation instanceof UpdateOperation) {
            $package = $operation->getTargetPackage();
        } elseif ($operation instanceof InstallOperation) {
            $package = $operation->getPackage();
        }

        if (null === $package) {
            return false;
        }

        return self::PACKAGE_NAME === $package->getName();
    }
}
