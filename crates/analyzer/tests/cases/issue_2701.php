<?php

declare(strict_types=1);

namespace Issue2701;

/**
 * @phpstan-type ReaderConfiguration array{
 *     file: string|null,
 *     service: string|null,
 *     separator: string,
 * }
 *
 * @phpstan-type ImportConfiguration array{
 *     base_dir: string,
 *     encoding: string|null,
 *     user?: ReaderConfiguration,
 * }
 */
final class Importer
{
    /** @param ImportConfiguration $config */
    public function nested(array $config): void
    {
        if (isset($config['user'])) {
            if (isset($config['user']['file'])) {
                $this->openFile($config['base_dir'] . '/' . $config['user']['file'], $config['user']['separator']);
            } elseif (isset($config['user']['service'])) {
                $this->getReaderService($config['user']['service']);
            }
        }
    }

    /** @param ImportConfiguration $config */
    public function afterEarlyReturn(array $config): void
    {
        if (!isset($config['user'])) {
            return;
        }

        if (isset($config['user']['file'])) {
            $this->openFile($config['user']['file'], $config['user']['separator']);
        }
    }

    /** @param ImportConfiguration $config */
    public function flat(array $config): void
    {
        if (isset($config['user']['file'])) {
            $this->openFile($config['user']['file'], $config['user']['separator']);
        }
    }

    /** @param ImportConfiguration $config */
    public function stillNullable(array $config): void
    {
        if (isset($config['user'])) {
            // @mago-expect analysis:possibly-null-argument
            $this->getReaderService($config['user']['service']);
        }
    }

    private function openFile(string $path, string $separator): void
    {
        echo $path . $separator;
    }

    private function getReaderService(string $service): void
    {
        echo $service;
    }
}
