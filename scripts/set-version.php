#!/usr/bin/env php
<?php

declare(strict_types=1);

namespace Mago\Scripts;

use RuntimeException;

use function count;
use function dirname;
use function explode;
use function file_get_contents;
use function file_put_contents;
use function fwrite;
use function implode;
use function is_file;
use function is_string;
use function preg_match;
use function preg_quote;
use function preg_replace;
use function rtrim;
use function sprintf;
use function str_replace;
use function trim;

use const STDERR;
use const STDOUT;

const VERSIONED_DOCUMENTATION_FILES = [
    'README.md',
    'docs/content/en/guide/configuration.md',
    'docs/content/en/guide/installation.md',
    'docs/content/en/guide/upgrading.md',
    'docs/content/en/recipes/docker.md',
    'docs/content/fr/guide/configuration.md',
    'docs/content/fr/guide/installation.md',
    'docs/content/fr/guide/upgrading.md',
    'docs/content/fr/recipes/docker.md',
    'docs/content/zh/guide/configuration.md',
    'docs/content/zh/guide/installation.md',
    'docs/content/zh/guide/upgrading.md',
    'docs/content/zh/recipes/docker.md',
];

function main(mixed $newVersion, int $argumentCount): int
{
    if (1 !== $argumentCount || !is_string($newVersion)) {
        fwrite(STDERR, "Usage: php scripts/set-version.php <version>\n");

        return 64;
    }

    if (1 !== preg_match('/^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$/D', $newVersion)) {
        fwrite(STDERR, sprintf("Invalid release version: %s\n", $newVersion));

        return 64;
    }

    $root = dirname(__DIR__);
    $manifestPath = $root . '/Cargo.toml';
    $manifest = namespace\read_file($manifestPath);
    $currentVersion = namespace\read_package_version($manifest);

    [$currentMajor, $currentMinor] = explode('.', $currentVersion);
    [$newMajor, $newMinor] = explode('.', $newVersion);

    if ($currentMajor !== $newMajor) {
        fwrite(STDERR, sprintf(
            "Major version bumps are not supported by this script: %s to %s.\n",
            $currentVersion,
            $newVersion,
        ));

        return 64;
    }

    if ($currentVersion === $newVersion) {
        fwrite(STDOUT, sprintf("Mago is already at version %s.\n", $newVersion));

        return 0;
    }

    $updates = [];
    $updates[$manifestPath] = namespace\update_manifest($manifest, $newVersion);

    $lockPath = $root . '/Cargo.lock';
    $updates[$lockPath] = namespace\update_lock(namespace\read_file($lockPath), $newVersion);

    foreach (VERSIONED_DOCUMENTATION_FILES as $relativePath) {
        $path = $root . '/' . $relativePath;
        $contents = namespace\read_file($path);
        $updated = str_replace($currentVersion, $newVersion, $contents);

        if ($currentMinor !== $newMinor) {
            $currentMinorVersion = $currentMajor . '.' . $currentMinor;
            $newMinorVersion = $newMajor . '.' . $newMinor;
            $pattern = sprintf('/(?<![0-9.])%s(?![0-9]|\.[0-9])/', preg_quote($currentMinorVersion, '/'));
            $updated = preg_replace($pattern, $newMinorVersion, $updated);
            if (null === $updated) {
                throw new RuntimeException(sprintf('Unable to update minor versions in %s.', $path));
            }
        }

        if ($updated !== $contents) {
            $updates[$path] = $updated;
        }
    }

    foreach ($updates as $path => $contents) {
        if (false === file_put_contents($path, $contents)) {
            throw new RuntimeException(sprintf('Unable to write %s.', $path));
        }
    }

    fwrite(STDOUT, sprintf("Updated Mago from %s to %s in %d files.\n", $currentVersion, $newVersion, count($updates)));

    return 0;
}

function read_file(string $path): string
{
    if (!is_file($path)) {
        throw new RuntimeException(sprintf('Required file does not exist: %s.', $path));
    }

    $contents = file_get_contents($path);
    if (false === $contents) {
        throw new RuntimeException(sprintf('Unable to read %s.', $path));
    }

    return $contents;
}

function read_package_version(string $manifest): string
{
    $section = null;
    foreach (explode("\n", $manifest) as $line) {
        $line = rtrim($line, "\r");
        $matches = [];
        if (1 === preg_match('/^\[([^]]+)]$/', trim($line), $matches)) {
            $section = $matches[1] ?? throw new RuntimeException('Unable to read a Cargo.toml section name.');

            continue;
        }

        $matches = [];
        if ('package' === $section && 1 === preg_match('/^version = "([^"]+)"$/', $line, $matches)) {
            return $matches[1] ?? throw new RuntimeException('Unable to read the root package version.');
        }
    }

    throw new RuntimeException('Unable to find the root package version in Cargo.toml.');
}

function update_manifest(string $manifest, string $newVersion): string
{
    $section = null;
    $packageUpdates = 0;
    $workspaceUpdates = 0;
    $dependencyUpdates = 0;
    $lines = explode("\n", $manifest);

    foreach ($lines as &$line) {
        $normalized = rtrim($line, "\r");
        $matches = [];
        if (1 === preg_match('/^\[([^]]+)]$/', trim($normalized), $matches)) {
            $section = $matches[1] ?? throw new RuntimeException('Unable to read a Cargo.toml section name.');

            continue;
        }

        $shouldUpdate =
            'package' === $section && 1 === preg_match('/^version = "[^"]+"$/', $normalized)
            || 'workspace' === $section && 1 === preg_match('/^package\.version = "[^"]+"$/', $normalized)
            || 'workspace.dependencies' === $section
            && 1 === preg_match('/^mago-[a-z0-9-]+ = \{[^}]*path = "[^"]+"[^}]*version = "[^"]+"[^}]*}$/', $normalized);

        if (!$shouldUpdate) {
            continue;
        }

        $updated = preg_replace('/version = "[^"]+"/', sprintf('version = "%s"', $newVersion), $line, 1);
        if (null === $updated) {
            throw new RuntimeException('Unable to update a version in Cargo.toml.');
        }

        $line = $updated;
        if ('package' === $section) {
            ++$packageUpdates;
        } elseif ('workspace' === $section) {
            ++$workspaceUpdates;
        } else {
            ++$dependencyUpdates;
        }
    }
    unset($line);

    if (1 !== $packageUpdates || 1 !== $workspaceUpdates || 0 === $dependencyUpdates) {
        throw new RuntimeException('Cargo.toml does not contain the expected Mago version declarations.');
    }

    return implode("\n", $lines);
}

function update_lock(string $lock, string $newVersion): string
{
    $insideMagoPackage = false;
    $replacements = 0;
    $lines = explode("\n", $lock);

    foreach ($lines as &$line) {
        $normalized = rtrim($line, "\r");
        if ('[[package]]' === $normalized) {
            $insideMagoPackage = false;

            continue;
        }

        if (1 === preg_match('/^name = "mago(?:-[^"]+)?"$/', $normalized)) {
            $insideMagoPackage = true;

            continue;
        }

        if (!$insideMagoPackage || 1 !== preg_match('/^version = "[^"]+"$/', $normalized)) {
            continue;
        }

        $line = str_replace($normalized, sprintf('version = "%s"', $newVersion), $line);
        $insideMagoPackage = false;
        ++$replacements;
    }
    unset($line);

    if (0 === $replacements) {
        throw new RuntimeException('Cargo.lock does not contain any Mago packages.');
    }

    return implode("\n", $lines);
}

try {
    exit(namespace\main($argv[1] ?? null, count($argv) - 1));
} catch (RuntimeException $exception) {
    fwrite(STDERR, $exception->getMessage() . "\n");

    exit(1);
}
