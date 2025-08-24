<?php

declare(strict_types=1);

namespace Mago;

use Composer\Command\BaseCommand;
use Composer\InstalledVersions;
use Composer\IO\ConsoleIO;
use Composer\Util\Filesystem;
use Composer\Util\Http\Response;
use Composer\Util\HttpDownloader;
use Composer\Util\ProcessExecutor;
use Symfony\Component\Console\Input\InputInterface;
use Symfony\Component\Console\Input\InputOption;
use Symfony\Component\Console\Output\OutputInterface;

use function is_string;

/**
 * @mago-expect lint:kan-defect
 * @mago-expect lint:cyclomatic-complexity
 * @mago-expect lint:halstead
 * @mago-expect lint:no-shorthand-ternary
 *
 * @mago-expect analysis:mixed-assignment
 */
final class InstallMagoBinaryCommand extends BaseCommand
{
    protected function configure(): void
    {
        $this->setName('mago:install-binary');
        $this->setDescription('Installs the mago binary for current platform.');
        $this->addOption('force', 'f', InputOption::VALUE_NONE, 'Force re-installation of the mago binary.');
    }

    protected function execute(InputInterface $input, OutputInterface $output): int
    {
        $composer = $this->requireComposer();
        $loop = $composer->getLoop();
        $downloader = $loop->getHttpDownloader();
        $io = $this->getIO();
        $process_executor = $loop->getProcessExecutor();
        assert(
            $process_executor instanceof ProcessExecutor,
            'Expecting a process executor, but none was found on the composer loop.',
        );

        try {
            $release = $this->detectMagoReleaseId($downloader);
            ['tag' => $release, 'downloads' => $downloads] = $this->buildAssetsMapForRelease($downloader, $release);
            [
                'platform' => $platform,
                'executable' => $executable,
                'storage_dir' => $storage_dir,
            ] = $this->detectPlatformInfo($process_executor, $release);
        } catch (\Throwable $e) {
            $io->error($e->getMessage());
            return self::FAILURE;
        }

        $download = $downloads[$storage_dir] ?? null;
        if (!$download) {
            $io->error("There is no mago {$release} download available for the current platform '{$platform}'.");
            return self::FAILURE;
        }

        $filesystem = new Filesystem($process_executor);
        $release_dir = __DIR__ . '/bin/' . $release;
        $target_dir = $release_dir . '/' . $storage_dir;
        $executable_path = $target_dir . '/' . $executable;
        $executable_platform_file = __DIR__ . '/bin/.platform';
        $executable_platform_content = $release . '/' . $storage_dir . '/' . $executable;

        $filesystem->ensureDirectoryExists($release_dir);
        if ($input->getOption('force')) {
            $filesystem->emptyDirectory($target_dir);
        }

        if (file_exists($executable_path)) {
            file_put_contents($executable_platform_file, $executable_platform_content);
            $io->write("Mago {$release} binaries for platform '{$platform}' already exist.");
            return self::SUCCESS;
        }

        $io->write("Downloading mago {$release} binary:");
        $io->write('');

        $io->write(" - {$download['file']}");

        $downloaded_file = $release_dir . '/' . $download['file'];
        $promise = $downloader
            ->addCopy($download['url'], $downloaded_file)
            ->then(static function (null|Response $response = null) use (
                $filesystem,
                $release_dir,
                $downloaded_file,
                $executable_platform_file,
                $executable_platform_content,
            ): null|Response {
                if (null === $response) {
                    return null;
                }

                $phar = new \PharData($downloaded_file);
                $phar->extractTo($release_dir);

                $filesystem->remove($downloaded_file);

                file_put_contents($executable_platform_file, $executable_platform_content);

                return $response;
            });

        $io->write('');
        $progress_bar = $io instanceof ConsoleIO ? $io->getProgressBar() : null;
        $loop->wait([$promise], $progress_bar);
        $io->write('');
        $io->write('');
        $io->write('Done!');

        return self::SUCCESS;
    }

    private function detectMagoReleaseId(HttpDownloader $httpDownloader): string
    {
        $version = InstalledVersions::getPrettyVersion(MagoPlugin::PACKAGE_NAME);

        $response = $httpDownloader->get($this->buildGithubApiUri('/releases?per_page=99999999999999999'));
        /** @var array<string, array{'tag_name': string, 'id': int|string, ...}> $json */
        $json = $response->decodeJson();

        foreach ($json as $release) {
            if ($release['tag_name'] === $version) {
                return (string) $release['id'];
            }
        }

        return 'latest';
    }

    /**
     * @return array{tag: string, downloads: array<string, array{file: string, url: string}>}
     */
    private function buildAssetsMapForRelease(HttpDownloader $httpDownloader, string $releaseId): array
    {
        $response = $httpDownloader->get($this->buildGithubApiUri('/releases/' . $releaseId));
        /**
         * @var array{
         *   'tag_name': string,
         *   'assets'?: list<array{browser_download_url: string, name: string}>
         * } $json
         */
        $json = $response->decodeJson();
        $assets = $json['assets'] ?? [];

        return [
            'tag' => $json['tag_name'],
            'downloads' => array_reduce(
                $assets,
                /**
                 * @param null|array<string, array{'file': string, 'url': string}> $downloadMap
                 * @param array{browser_download_url: string, name: string} $asset
                 *
                 * @return array<string, array{'file': string, 'url': string}>
                 */
                static function (null|array $downloadMap, array $asset): array {
                    $downloadMap = $downloadMap ?? [];

                    if (!str_ends_with($asset['name'], '.tar.gz') && !str_ends_with($asset['name'], '.zip')) {
                        return $downloadMap;
                    }

                    $platform = preg_replace('{^(.*)(\.tar\.gz|\.zip)$}', '$1', $asset['name']) ?? '';

                    $downloadMap[$platform] = [
                        'file' => $asset['name'],
                        'url' => $asset['browser_download_url'],
                    ];

                    return $downloadMap;
                },
                [],
            ),
        ];
    }

    private function buildGithubApiUri(string $path): string
    {
        return 'https://api.github.com/repos/carthage-software/mago' . $path;
    }

    /**
     * @mago-expect lint:best-practices/no-else-clause
     *
     * @return array{platform: string, executable: string, storage_dir: string}
     */
    private function detectPlatformInfo(ProcessExecutor $process_executor, string $version): array
    {
        $arch_name = strtolower(php_uname('m'));
        switch ($arch_name) {
            case 'x86_64':
            case 'amd64':
                $arch = 'x86_64';
                break;
            case 'arm64':
            case 'aarch64':
                $arch = 'aarch64';
                break;
            case 'armv7l':
                $arch = 'armv7';
                break;
            case 'i386':
            case 'i486':
            case 'i586':
            case 'i686':
                $arch = 'i686';
                break;
            case 'ppc':
                $arch = 'powerpc';
                break;
            case 'ppc64':
                $arch = 'powerpc64';
                break;
            case 'ppc64le':
                $arch = 'powerpc64le';
                break;
            case 's390x':
                $arch = 's390x';
                break;
            default:
                throw new \RuntimeException(
                    "Unsupported machine architecture: {$arch_name}. Please open an issue on GitHub.",
                );
        }

        $os = strtolower(php_uname('s'));
        $vendor = 'unknown';
        $os_suffix = '';
        $executable_extension = '';

        switch ($os) {
            case 'windows nt':
                $os = 'windows';
                $vendor = 'pc';
                $os_suffix = 'msvc';
                $executable_extension = '.exe';
                break;
            case 'darwin':
                $vendor = 'apple';
                break;
            case 'linux':
                if ($process_executor->execute('command -v ldd') === 0) {
                    $ldd_version = null;
                    $process_executor->execute('ldd --version 2>&1', $ldd_version);
                    if (is_string($ldd_version) && str_contains($ldd_version, 'musl')) {
                        switch ($arch) {
                            case 'x86_64':
                            case 'aarch64':
                            case 'i686':
                                $os_suffix = 'musl';
                                break;
                            case 'arm':
                            case 'armv7':
                                if (str_contains(file_get_contents('/proc/cpuinfo') ?: '', 'hard')) {
                                    $os_suffix = 'musleabihf';
                                } else {
                                    $os_suffix = 'musleabi';
                                }
                                break;
                            default:
                                throw new \RuntimeException("Unsupported architecture for musl: {$arch_name}");
                        }
                    } else {
                        switch ($arch) {
                            case 'x86_64':
                            case 'aarch64':
                            case 'i686':
                            case 'powerpc':
                            case 'powerpc64':
                            case 'powerpc64le':
                            case 's390x':
                                $os_suffix = 'gnu';
                                break;
                            case 'arm':
                            case 'armv7':
                                if (str_contains(file_get_contents('/proc/cpuinfo') ?: '', 'hard')) {
                                    $os_suffix = 'gnueabihf';
                                } else {
                                    $os_suffix = 'gnueabi';
                                }
                                break;
                            default:
                                throw new \RuntimeException("Unsupported architecture for glibc: {$arch_name}");
                        }
                    }
                } else {
                    $os_suffix = 'musl';
                }
                break;
            case 'freebsd':
                break;
            default:
                throw new \RuntimeException("Unsupported operating system: {$os}. Please open an issue on GitHub.");
        }

        $target_triple = $os_suffix ? "{$arch}-{$vendor}-{$os}-{$os_suffix}" : "{$arch}-{$vendor}-{$os}";
        $storage_dir = "mago-{$version}-{$target_triple}";
        $executable = "mago{$executable_extension}";

        return [
            'platform' => $target_triple,
            'storage_dir' => $storage_dir,
            'executable' => $executable,
        ];
    }
}
