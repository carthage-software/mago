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

const MANIFEST_VERSION_DECLARATIONS = [
    'package' => ['package', '/^version = "[^"]+"$/'],
    'workspace' => ['workspace', '/^package\.version = "[^"]+"$/'],
    'workspace.dependencies' => [
        'dependency',
        '/^mago-[a-z0-9-]+ = \{[^}]*path = "[^"]+"[^}]*version = "[^"]+"[^}]*}$/',
    ],
];

function set_version(mixed $newVersion, int $argumentCount): int
{
    $newVersion = namespace\validate_new_version($newVersion, $argumentCount);
    if (null === $newVersion) {
        return 64;
    }

    $root = dirname(__DIR__);
    $manifestPath = $root . '/Cargo.toml';
    $manifest = namespace\read_file($manifestPath);
    $currentVersion = namespace\read_package_version($manifest);

    if (!namespace\versions_share_major($currentVersion, $newVersion)) {
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

    $updates = namespace\prepare_updates($root, $manifest, $currentVersion, $newVersion);
    namespace\write_updates($updates);

    fwrite(STDOUT, sprintf("Updated Mago from %s to %s in %d files.\n", $currentVersion, $newVersion, count($updates)));

    return 0;
}

function validate_new_version(mixed $newVersion, int $argumentCount): ?string
{
    if (1 !== $argumentCount || !is_string($newVersion)) {
        fwrite(STDERR, "Usage: php scripts/set-version.php <version>\n");

        return null;
    }

    if (1 !== preg_match('/^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$/D', $newVersion)) {
        fwrite(STDERR, sprintf("Invalid release version: %s\n", $newVersion));

        return null;
    }

    return $newVersion;
}

function versions_share_major(string $currentVersion, string $newVersion): bool
{
    [$currentMajor] = explode('.', $currentVersion);
    [$newMajor] = explode('.', $newVersion);

    return $currentMajor === $newMajor;
}

function minor_version(string $version): string
{
    [$major, $minor] = explode('.', $version);

    return $major . '.' . $minor;
}

/** @return array<string, string> */
function prepare_updates(string $root, string $manifest, string $currentVersion, string $newVersion): array
{
    $manifestPath = $root . '/Cargo.toml';
    $lockPath = $root . '/Cargo.lock';

    return (
        [
            $manifestPath => namespace\update_manifest($manifest, $newVersion),
            $lockPath => namespace\update_lock(namespace\read_file($lockPath), $newVersion),
        ] + namespace\prepare_documentation_updates($root, $currentVersion, $newVersion)
    );
}

/** @return array<string, string> */
function prepare_documentation_updates(string $root, string $currentVersion, string $newVersion): array
{
    $updates = [];

    foreach (VERSIONED_DOCUMENTATION_FILES as $relativePath) {
        $path = $root . '/' . $relativePath;
        $contents = namespace\read_file($path);
        $updated = namespace\update_documentation($path, $contents, $currentVersion, $newVersion);

        if ($updated === $contents) {
            continue;
        }

        $updates[$path] = $updated;
    }

    return $updates;
}

function update_documentation(string $path, string $contents, string $currentVersion, string $newVersion): string
{
    $updated = str_replace($currentVersion, $newVersion, $contents);
    $currentMinorVersion = namespace\minor_version($currentVersion);
    $newMinorVersion = namespace\minor_version($newVersion);

    if ($currentMinorVersion === $newMinorVersion) {
        return $updated;
    }

    $pattern = sprintf('/(?<![0-9.])%s(?![0-9]|\.[0-9])/', preg_quote($currentMinorVersion, '/'));
    $updated = preg_replace($pattern, $newMinorVersion, $updated);
    if (null === $updated) {
        throw new RuntimeException(sprintf('Unable to update minor versions in %s.', $path));
    }

    return $updated;
}

/** @param array<string, string> $updates */
function write_updates(array $updates): void
{
    foreach ($updates as $path => $contents) {
        if (false === file_put_contents($path, $contents)) {
            throw new RuntimeException(sprintf('Unable to write %s.', $path));
        }
    }
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
        $nextSection = namespace\read_manifest_section($normalized);
        if (null !== $nextSection) {
            $section = $nextSection;

            continue;
        }

        $updateKind = namespace\get_manifest_update_kind($section, $normalized);
        if (null === $updateKind) {
            continue;
        }

        $line = namespace\update_manifest_line($line, $newVersion);

        if ('package' === $updateKind) {
            ++$packageUpdates;

            continue;
        }

        if ('workspace' === $updateKind) {
            ++$workspaceUpdates;

            continue;
        }

        ++$dependencyUpdates;
    }
    unset($line);

    namespace\validate_manifest_updates($packageUpdates, $workspaceUpdates, $dependencyUpdates);

    return implode("\n", $lines);
}

function read_manifest_section(string $line): ?string
{
    $matches = [];
    if (1 !== preg_match('/^\[([^]]+)]$/', trim($line), $matches)) {
        return null;
    }

    return $matches[1] ?? throw new RuntimeException('Unable to read a Cargo.toml section name.');
}

function get_manifest_update_kind(?string $section, string $line): ?string
{
    if (null === $section) {
        return null;
    }

    $declaration = MANIFEST_VERSION_DECLARATIONS[$section] ?? null;
    if (null === $declaration) {
        return null;
    }

    [$kind, $pattern] = $declaration;
    if (1 !== preg_match($pattern, $line)) {
        return null;
    }

    return $kind;
}

function update_manifest_line(string $line, string $newVersion): string
{
    $updated = preg_replace('/version = "[^"]+"/', sprintf('version = "%s"', $newVersion), $line, 1);
    if (null === $updated) {
        throw new RuntimeException('Unable to update a version in Cargo.toml.');
    }

    return $updated;
}

function validate_manifest_updates(int $packageUpdates, int $workspaceUpdates, int $dependencyUpdates): void
{
    if (1 !== $packageUpdates) {
        throw new RuntimeException('Cargo.toml does not contain the expected Mago version declarations.');
    }

    if (1 !== $workspaceUpdates) {
        throw new RuntimeException('Cargo.toml does not contain the expected Mago version declarations.');
    }

    if (0 === $dependencyUpdates) {
        throw new RuntimeException('Cargo.toml does not contain the expected Mago version declarations.');
    }
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
    exit(namespace\set_version($argv[1] ?? null, count($argv) - 1));
} catch (RuntimeException $exception) {
    fwrite(STDERR, $exception->getMessage() . "\n");

    exit(1);
}
