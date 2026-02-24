<?php

declare(strict_types=1);

namespace Mago\Internal;

use Composer\InstalledVersions;
use PharData;
use RuntimeException;
use ZipArchive;

use function array_map;
use function chmod;
use function curl_error;
use function curl_exec;
use function curl_getinfo;
use function curl_init;
use function curl_setopt;
use function escapeshellarg;
use function escapeshellcmd;
use function extension_loaded;
use function fclose;
use function file_exists;
use function file_get_contents;
use function file_put_contents;
use function fopen;
use function fprintf;
use function fwrite;
use function implode;
use function ini_get;
use function is_dir;
use function is_resource;
use function mkdir;
use function number_format;
use function php_uname;
use function proc_close;
use function proc_get_status;
use function proc_open;
use function shell_exec;
use function str_contains;
use function strtolower;
use function trim;
use function unlink;

use const CURLINFO_HTTP_CODE;
use const CURLOPT_FILE;
use const CURLOPT_FOLLOWLOCATION;
use const CURLOPT_NOPROGRESS;
use const CURLOPT_PROGRESSFUNCTION;
use const STDERR;

/**
 * Get the installed mago version from Composer metadata.
 *
 * @throws RuntimeException If the version cannot be determined.
 *
 * @return string The package version (e.g., "1.10.0").
 */
function get_version(): string
{
    $version = InstalledVersions::getPrettyVersion('carthage-software/mago');
    if ($version === null) {
        throw new RuntimeException('Could not determine mago package version.');
    }

    return $version;
}

/**
 * Detect the CPU architecture from the system.
 *
 * Maps the raw `php_uname('m')` value to a normalized Rust target architecture name.
 * Only architectures with pre-built release binaries are supported.
 *
 * Supported: x86_64, aarch64, armv7, arm (v5/v6).
 *
 * @throws RuntimeException If the architecture is not supported.
 *
 * @return string Normalized architecture (e.g., "x86_64", "aarch64", "armv7", "arm").
 */
function detect_architecture(): string
{
    $raw = strtolower(php_uname('m'));

    return match ($raw) {
        'x86_64', 'amd64' => 'x86_64',
        'arm64', 'aarch64' => 'aarch64',
        'armv7l' => 'armv7',
        'armv6l', 'armv5tel', 'armv5l' => 'arm',
        default => throw new RuntimeException(
            "Unsupported architecture: {$raw}. Pre-built binaries are available for x86_64, aarch64, armv7, and arm. "
            . 'For other architectures, compile mago from source: https://github.com/carthage-software/mago',
        ),
    };
}

/**
 * Detect the C library variant on Linux (glibc or musl).
 *
 * Uses `ldd --version` output to distinguish between glibc and musl.
 * Falls back to musl if `ldd` is not found (common in minimal containers).
 *
 * @return string "musl" or "gnu".
 */
function detect_linux_libc(): string
{
    $lddPath = trim((string) shell_exec('command -v ldd 2>/dev/null'));
    if ($lddPath === '') {
        return 'musl';
    }

    $lddVersion = (string) shell_exec('ldd --version 2>&1');

    return str_contains($lddVersion, 'musl') ? 'musl' : 'gnu';
}

/**
 * Detect the ARM float ABI from /proc/cpuinfo.
 *
 * Checks whether the CPU supports hardware floating point (hard float).
 *
 * @return bool True if the system uses hard float ABI.
 */
function detect_arm_hard_float(): bool
{
    return str_contains((string) file_get_contents('/proc/cpuinfo'), 'hard');
}

/**
 * Build the Linux OS suffix for the target triple.
 *
 * Combines the C library variant (glibc/musl) with the appropriate ABI suffix
 * for the given architecture. Only combinations with pre-built binaries are supported.
 *
 * Released Linux targets:
 *   x86_64:  gnu, musl
 *   aarch64: gnu, musl
 *   armv7:   gnueabihf, musleabihf (hard float only)
 *   arm:     gnueabi, gnueabihf, musleabi, musleabihf
 *
 * @param string $architecture Normalized architecture name.
 * @param string $libc C library variant ("gnu" or "musl").
 *
 * @throws RuntimeException If the architecture/libc combination has no pre-built binary.
 *
 * @return string The OS suffix (e.g., "gnu", "musl", "gnueabihf", "musleabi").
 */
function build_linux_suffix(string $architecture, string $libc): string
{
    $hardFloat = match ($architecture) {
        'arm', 'armv7' => namespace\detect_arm_hard_float(),
        default => false,
    };

    if ($libc === 'musl') {
        return match ($architecture) {
            'x86_64', 'aarch64' => 'musl',
            'armv7' => 'musleabihf',
            'arm' => $hardFloat ? 'musleabihf' : 'musleabi',
            default => throw new RuntimeException("No pre-built musl binary for architecture: {$architecture}"),
        };
    }

    return match ($architecture) {
        'x86_64', 'aarch64' => 'gnu',
        'armv7' => 'gnueabihf',
        'arm' => $hardFloat ? 'gnueabihf' : 'gnueabi',
        default => throw new RuntimeException("No pre-built glibc binary for architecture: {$architecture}"),
    };
}

/**
 * Detect the operating system and build platform metadata.
 *
 * Returns all platform-specific values needed to construct the download URL
 * and locate the extracted binary. Only OS/architecture combinations with
 * pre-built release binaries are supported.
 *
 * Released OS targets:
 *   Windows: x86_64 only (msvc)
 *   macOS:   x86_64, aarch64
 *   Linux:   x86_64, aarch64, armv7, arm
 *   FreeBSD: x86_64 only
 *
 * @param string $architecture Normalized architecture name.
 *
 * @throws RuntimeException If the OS/architecture combination is not supported.
 *
 * @return array{os: string, vendor: string, suffix: string, extension: string}
 *   - os: Normalized OS name (e.g., "linux", "darwin", "windows")
 *   - vendor: Target vendor (e.g., "unknown", "apple", "pc")
 *   - suffix: OS/ABI suffix for the target triple (e.g., "gnu", "musl", "msvc")
 *   - extension: Executable file extension (e.g., "", ".exe")
 */
function detect_platform(string $architecture): array
{
    $os = strtolower(php_uname('s'));

    return match ($os) {
        'windows nt' => match ($architecture) {
            'x86_64' => [
                'os' => 'windows',
                'vendor' => 'pc',
                'suffix' => 'msvc',
                'extension' => '.exe',
            ],
            default => throw new RuntimeException(
                "No pre-built Windows binary for architecture: {$architecture}. Only x86_64 is supported.",
            ),
        },
        'darwin' => match ($architecture) {
            'x86_64', 'aarch64' => [
                'os' => 'darwin',
                'vendor' => 'apple',
                'suffix' => '',
                'extension' => '',
            ],
            default => throw new RuntimeException(
                "No pre-built macOS binary for architecture: {$architecture}. Only x86_64 and aarch64 are supported.",
            ),
        },
        'linux' => [
            'os' => 'linux',
            'vendor' => 'unknown',
            'suffix' => namespace\build_linux_suffix($architecture, namespace\detect_linux_libc()),
            'extension' => '',
        ],
        'freebsd' => match ($architecture) {
            'x86_64' => [
                'os' => 'freebsd',
                'vendor' => 'unknown',
                'suffix' => '',
                'extension' => '',
            ],
            default => throw new RuntimeException(
                "No pre-built FreeBSD binary for architecture: {$architecture}. Only x86_64 is supported.",
            ),
        },
        default => throw new RuntimeException(
            "Unsupported operating system: {$os}. Pre-built binaries are available for Windows, macOS, Linux, and FreeBSD. "
            . 'For other platforms, compile mago from source: https://github.com/carthage-software/mago',
        ),
    };
}

/**
 * Build the Rust target triple from platform components.
 *
 * @param string $arch Architecture (e.g., "x86_64").
 * @param string $vendor Vendor (e.g., "unknown", "apple").
 * @param string $os OS name (e.g., "linux", "darwin").
 * @param string $suffix ABI suffix (e.g., "gnu", "musl", "msvc", or "").
 *
 * @return string The target triple (e.g., "x86_64-unknown-linux-gnu", "aarch64-apple-darwin").
 */
function build_target_triple(string $arch, string $vendor, string $os, string $suffix): string
{
    if ($suffix !== '') {
        return "{$arch}-{$vendor}-{$os}-{$suffix}";
    }

    return "{$arch}-{$vendor}-{$os}";
}

/**
 * Get the archive file extension for the given platform.
 *
 * Windows MSVC builds are packaged as `.zip`, all other platforms use `.tar.gz`.
 *
 * @return string ".zip" or ".tar.gz".
 */
function get_archive_extension(string $os, string $suffix): string
{
    return $os === 'windows' && $suffix === 'msvc' ? '.zip' : '.tar.gz';
}

/**
 * Build the GitHub release download URL for a given version and target.
 *
 * @param string $version Package version (e.g., "1.10.0").
 * @param string $storageDir Directory name inside the archive (e.g., "mago-1.10.0-x86_64-unknown-linux-gnu").
 * @param string $archiveExtension Archive extension (".zip" or ".tar.gz").
 *
 * @return string Full download URL.
 */
function build_download_url(string $version, string $storageDir, string $archiveExtension): string
{
    return "https://github.com/carthage-software/mago/releases/download/{$version}/{$storageDir}{$archiveExtension}";
}

/**
 * Download a file from a URL using the best available method.
 *
 * Prefers the curl extension (with progress display) and falls back to
 * `file_get_contents` when `allow_url_fopen` is enabled.
 *
 * @throws RuntimeException If the download fails or no download method is available.
 */
function download(string $url, string $destination): void
{
    if (extension_loaded('curl')) {
        namespace\download_with_curl($url, $destination);

        return;
    }

    if (ini_get('allow_url_fopen')) {
        namespace\download_with_fopen($url, $destination);

        return;
    }

    throw new RuntimeException(
        'Unable to download mago binary. Either install the PHP curl extension or set allow_url_fopen=1 in php.ini.',
    );
}

/**
 * Download a file using the curl extension with a progress bar.
 *
 * @throws RuntimeException If the download fails or the server returns an error status.
 */
function download_with_curl(string $url, string $destination): void
{
    $ch = curl_init($url);
    $fh = fopen($destination, 'w');
    curl_setopt($ch, CURLOPT_FOLLOWLOCATION, true);
    curl_setopt($ch, CURLOPT_FILE, $fh);
    curl_setopt($ch, CURLOPT_NOPROGRESS, false);
    curl_setopt($ch, CURLOPT_PROGRESSFUNCTION, function (mixed $_resource, int $dlSize, int $dlNow): int {
        if ($dlSize > 0) {
            $pct = (int) (($dlNow / $dlSize) * 100);
            $dlMb = number_format($dlNow / 1_048_576, 1);
            $totalMb = number_format($dlSize / 1_048_576, 1);
            fprintf(STDERR, "\r  %s / %s MB (%d%%)", $dlMb, $totalMb, $pct);
        }

        return 0;
    });

    $success = curl_exec($ch);
    /** @var int<100, 599> */
    $statusCode = curl_getinfo($ch, CURLINFO_HTTP_CODE);
    $error = curl_error($ch);
    fclose($fh);

    if (!$success || $statusCode >= 400) {
        unlink($destination);

        throw new RuntimeException("Failed to download mago binary (HTTP {$statusCode}): {$error}\nURL: {$url}");
    }

    fprintf(STDERR, "\n");
}

/**
 * Download a file using `file_get_contents` (requires `allow_url_fopen`).
 *
 * @throws RuntimeException If the download fails.
 */
function download_with_fopen(string $url, string $destination): void
{
    $contents = file_get_contents($url);
    if ($contents === false) {
        throw new RuntimeException("Failed to download mago binary.\nURL: {$url}");
    }

    file_put_contents($destination, $contents);
}

/**
 * Extract an archive to a destination directory.
 *
 * Supports `.zip` (via ZipArchive) and `.tar.gz` (via PharData).
 *
 * @throws RuntimeException If the archive cannot be opened or extracted.
 */
function extract_archive(string $archiveFile, string $destination, string $archiveExtension): void
{
    if ($archiveExtension === '.zip') {
        $zip = new ZipArchive();
        if ($zip->open($archiveFile) !== true) {
            unlink($archiveFile);

            throw new RuntimeException('Failed to open zip archive.');
        }

        $zip->extractTo($destination);
        $zip->close();

        unlink($archiveFile);
        return;
    }

    $phar = new PharData($archiveFile);
    $phar->extractTo($destination);
    unlink($archiveFile);
}

/**
 * Ensure the mago binary is available, downloading it if necessary.
 *
 * Checks if the binary already exists at the expected path. If not, downloads
 * the appropriate release archive, extracts it, and sets executable permissions.
 *
 * @param string $version Package version.
 * @param string $triple Target triple (e.g., "x86_64-unknown-linux-gnu").
 * @param string $executableExtension Executable extension ("" or ".exe").
 * @param string $archiveExtension Archive extension (".zip" or ".tar.gz").
 * @param string $binDir Base directory for storing binaries.
 *
 * @throws RuntimeException If the download, extraction, or binary verification fails.
 *
 * @return string Path to the mago executable.
 */
function ensure_binary(
    string $version,
    string $triple,
    string $executableExtension,
    string $archiveExtension,
    string $binDir,
): string {
    $storageDir = "mago-{$version}-{$triple}";
    $releaseDir = "{$binDir}/{$version}";
    $executablePath = "{$releaseDir}/{$storageDir}/mago{$executableExtension}";

    if (file_exists($executablePath)) {
        return $executablePath;
    }

    $archiveFile = "{$releaseDir}/{$storageDir}{$archiveExtension}";
    $url = namespace\build_download_url($version, $storageDir, $archiveExtension);

    if (!is_dir($releaseDir)) {
        mkdir($releaseDir, 0o755, true);
    }

    fprintf(STDERR, "Downloading mago %s for %s...\n", $version, $triple);
    namespace\download($url, $archiveFile);
    fprintf(STDERR, "Downloaded.\n");

    namespace\extract_archive($archiveFile, $releaseDir, $archiveExtension);

    if (!file_exists($executablePath)) {
        throw new RuntimeException("Expected binary not found after extraction at {$executablePath}");
    }

    // Ensure binary is executable on Unix
    if ($executableExtension === '') {
        chmod($executablePath, 0o755);
    }

    return $executablePath;
}

/**
 * Execute the mago binary, forwarding stdin/stdout/stderr.
 *
 * This function does not return; it exits with the binary's exit code.
 *
 * @param string $executablePath Path to the mago binary.
 * @param list<string> $args Command-line arguments to pass.
 *
 * @return never
 *
 * @mago-expect lint:no-error-control-operator
 */
function execute(string $executablePath, array $args): never
{
    $command = escapeshellcmd($executablePath);
    if ($args !== []) {
        $command .= ' ' . implode(' ', array_map(escapeshellarg(...), $args));
    }

    $pipes = [];
    $process = @proc_open(
        $command,
        [
            0 => ['file', 'php://stdin', 'r'],
            1 => ['file', 'php://stdout', 'w'],
            2 => ['file', 'php://stderr', 'w'],
        ],
        $pipes,
    );

    if (!is_resource($process)) {
        fwrite(STDERR, "Error: Unable to start mago process.\n");
        exit(1);
    }

    do {
        $status = proc_get_status($process);
    } while ($status['running']);

    $exitCode = $status['exitcode'];
    proc_close($process);

    exit($exitCode);
}
