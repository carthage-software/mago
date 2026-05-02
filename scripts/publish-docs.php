#!/usr/bin/env php
<?php

declare(strict_types=1);

namespace Mago\Scripts;

use RuntimeException;

use function array_filter;
use function array_key_last;
use function array_slice;
use function array_values;
use function copy;
use function count;
use function dirname;
use function explode;
use function file_get_contents;
use function file_put_contents;
use function fwrite;
use function is_array;
use function is_bool;
use function is_dir;
use function is_file;
use function is_string;
use function json_decode;
use function json_encode;
use function ltrim;
use function mkdir;
use function preg_match;
use function rmdir;
use function scandir;
use function sprintf;
use function str_ends_with;
use function str_starts_with;
use function stripslashes;
use function substr;
use function trim;
use function unlink;
use function usort;

use const JSON_PRETTY_PRINT;
use const JSON_THROW_ON_ERROR;
use const JSON_UNESCAPED_SLASHES;
use const STDERR;

/**
 * Stage the freshly built docs onto the gh-pages worktree.
 *
 * Steps, in order:
 *  - mirror `docs/dist/` into `<gh-pages>/<version>/` (replacing any prior
 *    contents of that version's directory),
 *  - on a `main` build, refresh the cross-version `sponsors.json`,
 *  - keep the custom-domain `CNAME` in lock-step with `docs/`,
 *  - copy redirect stubs from `dist/` into the worktree root,
 *  - update `versions.json` with the current build,
 *  - mirror the highest stable build to `latest/`,
 *  - write the root `index.html` redirect.
 *
 * Usage: php scripts/publish-docs.php <version> <docs-dir> <gh-pages-dir>
 *
 * @param list<string> $arguments
 */
function main(array $arguments): int
{
    if (3 !== count($arguments)) {
        fwrite(STDERR, "usage: publish-docs.php <version> <docs-dir> <gh-pages-dir>\n");
        return 64;
    }

    [$version, $docs, $ghPages] = $arguments;
    $built = $docs . '/dist';

    namespace\mirror_version($built, $ghPages, $version);
    namespace\refresh_sponsors($docs, $ghPages, $version);
    namespace\refresh_cname($docs, $ghPages);
    namespace\refresh_root_seo_files($built, $ghPages, $version);
    namespace\copy_redirect_stubs($docs . '/redirects.toml', $built, $ghPages);

    $versions = namespace\update_versions_file($ghPages . '/versions.json', $version);
    $latestStable = namespace\latest_stable_version($versions);
    namespace\mirror_latest($ghPages, $latestStable);
    namespace\write_root_redirect($ghPages, null !== $latestStable ? 'latest' : 'main');

    return 0;
}

function mirror_version(string $built, string $ghPages, string $version): void
{
    // The Rust builder writes per-version content at `dist/<version>/`, with
    // redirect stubs and meta files (sitemap, robots, versions.json) at
    // `dist/`'s root. We only mirror the version-scoped subtree onto
    // `gh-pages/<version>/`; the root files are handled by the dedicated
    // copy_redirect_stubs / refresh_sponsors / write_root_redirect helpers.
    $source = $built . '/' . $version;
    if (!is_dir($source)) {
        throw new RuntimeException(sprintf('built docs version directory not found at %s', $source));
    }

    $destination = $ghPages . '/' . $version;
    namespace\remove_directory($destination);
    namespace\copy_directory($source, $destination);
}

function refresh_sponsors(string $docs, string $ghPages, string $version): void
{
    if ('main' !== $version) {
        return;
    }

    $source = $docs . '/sponsors.json';
    if (!is_file($source)) {
        return;
    }

    copy($source, $ghPages . '/sponsors.json');
}

function refresh_cname(string $docs, string $ghPages): void
{
    $source = $docs . '/CNAME';
    if (!is_file($source)) {
        return;
    }

    copy($source, $ghPages . '/CNAME');
}

/**
 * Copy site-wide SEO files (sitemap.xml, robots.txt) onto the gh-pages
 * root. Only main-branch builds touch them so a tag deploy can't roll the
 * sitemap back to a stale state.
 */
function refresh_root_seo_files(string $built, string $ghPages, string $version): void
{
    if ('main' !== $version) {
        return;
    }

    foreach (['sitemap.xml', 'robots.txt'] as $name) {
        $source = $built . '/' . $name;
        if (!is_file($source)) {
            continue;
        }

        copy($source, $ghPages . '/' . $name);
    }
}

function copy_redirect_stubs(string $redirectsToml, string $built, string $ghPages): void
{
    $contents = file_get_contents($redirectsToml);
    if (false === $contents) {
        return;
    }

    foreach (namespace\parse_redirect_array($contents) as $redirect) {
        $relative = namespace\redirect_target_path($redirect['from']);
        $source = $built . '/' . $relative;
        $destination = $ghPages . '/' . $relative;
        if (!is_file($source)) {
            continue;
        }

        namespace\ensure_directory(dirname($destination));
        copy($source, $destination);
    }
}

function redirect_target_path(string $from): string
{
    $trimmed = ltrim($from, '/');
    if ('' === $trimmed) {
        return 'index.html';
    }
    if (str_ends_with($trimmed, '.html')) {
        return $trimmed;
    }

    return $trimmed . '/index.html';
}

/**
 * @return list<array{from: string}>
 */
function parse_redirect_array(string $contents): array
{
    /** @var list<array{from: string}> $entries */
    $entries = [];
    /** @var null|array<string, scalar|null> $current */
    $current = null;
    foreach (explode("\n", $contents) as $rawLine) {
        $line = trim($rawLine);
        if ('' === $line || str_starts_with($line, '#')) {
            continue;
        }

        if ('[[redirect]]' === $line) {
            namespace\push_redirect_entry($entries, $current);
            $current = [];
            continue;
        }

        if (null === $current) {
            continue;
        }

        $pair = namespace\parse_toml_pair($line);
        if (null === $pair) {
            continue;
        }

        $current[$pair[0]] = $pair[1];
    }

    namespace\push_redirect_entry($entries, $current);

    return $entries;
}

/**
 * @param list<array{from: string}> $entries
 * @param null|array<string, scalar|null> $current
 *
 * @param-out list<array{from: string}> $entries
 */
function push_redirect_entry(array &$entries, ?array $current): void
{
    if (null === $current) {
        return;
    }

    $from = $current['from'] ?? null;
    if (!is_string($from)) {
        return;
    }

    $entries[] = ['from' => $from];
}

/**
 * @return null|array{0: string, 1: scalar|null}
 */
function parse_toml_pair(string $line): ?array
{
    $matches = [];
    if (1 !== preg_match('/^([A-Za-z0-9_]+)\s*=\s*(.+)$/', $line, $matches)) {
        return null;
    }

    $key = $matches[1] ?? '';
    $value = $matches[2] ?? '';
    if ('' === $key) {
        return null;
    }

    return [$key, namespace\decode_toml_scalar(trim($value))];
}

function decode_toml_scalar(string $value): string|bool|null
{
    if ('true' === $value) {
        return true;
    }
    if ('false' === $value) {
        return false;
    }
    if (str_starts_with($value, '"') && str_ends_with($value, '"')) {
        return stripslashes(substr($value, 1, -1));
    }
    if (str_starts_with($value, "'") && str_ends_with($value, "'")) {
        return substr($value, 1, -1);
    }

    return null;
}

/**
 * @return list<array{id: string, label: string, stable: bool, paths: list<string>}>
 */
function update_versions_file(string $path, string $version): array
{
    $versions = namespace\read_versions_file($path);
    $stable = 1 === preg_match('/^\d+\.\d+\.\d+$/', $version);
    $label = $version;

    $updated = false;
    foreach ($versions as $index => $entry) {
        if ($entry['id'] !== $version) {
            continue;
        }
        $versions[$index]['label'] = $label;
        $versions[$index]['stable'] = $stable;
        $updated = true;
        break;
    }

    if (!$updated) {
        $versions[] = ['id' => $version, 'label' => $label, 'stable' => $stable, 'paths' => []];
    }

    file_put_contents(
        $path,
        json_encode(['versions' => $versions], JSON_PRETTY_PRINT | JSON_UNESCAPED_SLASHES | JSON_THROW_ON_ERROR) . "\n",
    );

    return $versions;
}

/**
 * @return list<array{id: string, label: string, stable: bool, paths: list<string>}>
 */
function read_versions_file(string $path): array
{
    if (!is_file($path)) {
        return [];
    }

    $raw = file_get_contents($path);
    if (false === $raw) {
        return [];
    }

    /** @var array{versions?: list<array<string, mixed>>} $decoded */
    $decoded = json_decode($raw, true, 512, JSON_THROW_ON_ERROR);
    $entries = $decoded['versions'] ?? [];

    /** @var list<array{id: string, label: string, stable: bool, paths: list<string>}> $normalised */
    $normalised = [];
    foreach ($entries as $entry) {
        $version = namespace\normalise_version_entry($entry);
        if (null !== $version) {
            $normalised[] = $version;
        }
    }

    return $normalised;
}

/**
 * @mago-expect lint:cyclomatic-complexity
 *
 * @param array<string, mixed> $entry
 *
 * @return null|array{id: string, label: string, stable: bool, paths: list<string>}
 */
function normalise_version_entry(array $entry): ?array
{
    /** @var mixed $id */
    $id = $entry['id'] ?? null;
    /** @var mixed $label */
    $label = $entry['label'] ?? null;
    if (!is_string($id) || !is_string($label)) {
        return null;
    }

    /** @var mixed $stable */
    $stable = $entry['stable'] ?? false;
    return [
        'id' => $id,
        'label' => $label,
        'stable' => is_bool($stable) && $stable,
        'paths' => namespace\normalise_path_list($entry['paths'] ?? []),
    ];
}

/**
 * @return list<string>
 */
function normalise_path_list(mixed $paths): array
{
    if (!is_array($paths)) {
        return [];
    }

    /** @var list<string> $result */
    $result = [];
    /** @var mixed $path */
    foreach ($paths as $path) {
        if (!is_string($path)) {
            continue;
        }

        $result[] = $path;
    }

    return $result;
}

/**
 * @param list<array{id: string, label: string, stable: bool, paths: list<string>}> $versions
 */
function latest_stable_version(array $versions): ?string
{
    $stable = array_values(array_filter(
        $versions,
        /**
         * @param array{id: string, label: string, stable: bool, paths: list<string>} $entry
         */
        static fn(array $entry): bool => $entry['stable'],
    ));
    if ([] === $stable) {
        return null;
    }

    usort(
        $stable,
        /**
         * @param array{id: string, label: string, stable: bool, paths: list<string>} $a
         * @param array{id: string, label: string, stable: bool, paths: list<string>} $b
         */
        static fn(array $a, array $b): int => namespace\semver_compare($a['id'], $b['id']),
    );

    $last = $stable[array_key_last($stable)];

    return $last['id'];
}

function semver_compare(string $a, string $b): int
{
    return namespace\semver_tuple($a) <=> namespace\semver_tuple($b);
}

/**
 * @return array{0: int, 1: int, 2: int}
 */
function semver_tuple(string $id): array
{
    $matches = [];
    if (1 !== preg_match('/^(\d+)\.(\d+)\.(\d+)$/', $id, $matches)) {
        return [-1, -1, -1];
    }

    return [(int) ($matches[1] ?? '0'), (int) ($matches[2] ?? '0'), (int) ($matches[3] ?? '0')];
}

function mirror_latest(string $ghPages, ?string $latestStable): void
{
    $latestDir = $ghPages . '/latest';
    namespace\remove_directory($latestDir);
    if (null === $latestStable) {
        return;
    }

    $source = $ghPages . '/' . $latestStable;
    if (!is_dir($source)) {
        return;
    }

    namespace\copy_directory($source, $latestDir);
}

function write_root_redirect(string $ghPages, string $target): void
{
    $body = sprintf(<<<'HTML'
        <!doctype html>
        <html lang="en">
        <head>
          <meta charset="utf-8">
          <title>Mago Documentation</title>
          <meta name="robots" content="noindex">
          <script>
            var target = '/%1$s/en/' + location.search + location.hash;
            location.replace(target);
          </script>
          <meta http-equiv="refresh" content="0; url=/%1$s/en/">
        </head>
        <body>
          <p>Redirecting to <a href="/%1$s/en/">/%1$s/en/</a>.</p>
        </body>
        </html>

        HTML, $target);

    file_put_contents($ghPages . '/index.html', $body);
}

function ensure_directory(string $path): void
{
    if (is_dir($path)) {
        return;
    }
    if (!mkdir($path, 0o755, true) && !is_dir($path)) {
        throw new RuntimeException(sprintf('failed to create directory %s', $path));
    }
}

function copy_directory(string $source, string $destination): void
{
    namespace\ensure_directory($destination);
    foreach (namespace\directory_entries($source) as $entry) {
        $sourceEntry = $source . '/' . $entry;
        $destinationEntry = $destination . '/' . $entry;
        if (is_dir($sourceEntry)) {
            namespace\copy_directory($sourceEntry, $destinationEntry);
            continue;
        }

        copy($sourceEntry, $destinationEntry);
    }
}

function remove_directory(string $path): void
{
    if (!is_dir($path)) {
        return;
    }

    foreach (namespace\directory_entries($path) as $entry) {
        $child = $path . '/' . $entry;
        if (is_dir($child)) {
            namespace\remove_directory($child);
            continue;
        }

        unlink($child);
    }

    rmdir($path);
}

/**
 * @return list<string>
 */
function directory_entries(string $path): array
{
    $entries = scandir($path);
    if (false === $entries) {
        return [];
    }

    /** @var list<string> $filtered */
    $filtered = [];
    foreach ($entries as $entry) {
        if ('.' === $entry || '..' === $entry) {
            continue;
        }
        $filtered[] = $entry;
    }

    return $filtered;
}

$arguments = array_values(array_slice($argv, 1));

exit(namespace\main($arguments));
