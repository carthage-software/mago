<?php

declare(strict_types=1);

namespace Mago;

use Composer\Command\BaseCommand;
use Composer\Composer;
use Composer\InstalledVersions;
use Composer\IO\ConsoleIO;
use Composer\Util\Filesystem;
use Composer\Util\Http\Response;
use Composer\Util\HttpDownloader;
use Symfony\Component\Console\Input\InputInterface;
use Symfony\Component\Console\Output\OutputInterface;

final class InstallMagoAssetsCommand extends BaseCommand
{
    protected function configure(): void
    {
        $this->setName('mago:install-assets');
        $this->setDescription('Installs the mago binaries for currently configured versions and platforms.');
    }

    protected function execute(InputInterface $input, OutputInterface $output): int
    {
        $composer = $this->requireComposer();
        $loop = $composer->getLoop();
        $downloader = $loop->getHttpDownloader();
        $io = $this->getIO();
        $config = $this->getMagoConfig($composer);

        $release = $this->detectMagoReleaseId($downloader);
        ['tag' => $release, 'downloads' => $downloads] = $this->buildAssetsMapForRelease(
            $downloader,
            $release,
            $config,
        );

        $filesystem = new Filesystem($loop->getProcessExecutor());
        $target_dir = __DIR__ . '/bin/' . $release;
        $filesystem->emptyDirectory($target_dir, ensureDirectoryExists: true);

        $io->write("Downloading mago {$release} binaries:");
        $io->write('');

        $promises = [];
        foreach ($downloads as $name => $url) {
            $io->write(" - {$name}");

            $target_file = $target_dir . '/' . $name;

            $promises[] = $downloader->addCopy($url, $target_file)->then(static function (Response $response) use (
                $filesystem,
                $target_dir,
                $target_file,
            ): Response {
                $phar = new \PharData($target_file);
                $phar->extractTo($target_dir);

                $filesystem->remove($target_file);

                return $response;
            });
        }

        file_put_contents(__DIR__ . '/bin/version', $release);

        $io->write('');
        $progress_bar = ($io instanceof ConsoleIO) ? $io->getProgressBar() : null;
        $loop->wait($promises, $progress_bar);
        $io->write('');
        $io->write('');
        $io->write('Done!');

        return self::SUCCESS;
    }

    private function detectMagoReleaseId(HttpDownloader $httpDownloader): string
    {
        $version = InstalledVersions::getVersion(MagoPlugin::PACKAGE_NAME);

        $response = $httpDownloader->get($this->buildGithubApiUri('/releases?per_page=99999999999999999'));
        $json = $response->decodeJson();

        foreach ($json as $release) {
            if ($release['tag_name'] === $version) {
                return $release['id'];
            }
        }

        return 'latest';
    }

    /**
     * @param array{platforms: list<string>} $config
     * @return array{tag: string, downloads: array<string, string>}
     */
    private function buildAssetsMapForRelease(HttpDownloader $httpDownloader, string $releaseId, array $config): array
    {
        $response = $httpDownloader->get($this->buildGithubApiUri('/releases/' . $releaseId));
        $json = $response->decodeJson();
        $platforms = $config['platforms'];

        return [
            'tag' => $json['tag_name'],
            'downloads' => array_reduce(
                $json['assets'] ?? [],
                /**
                 * @param array<string, string> $downloadMap
                 * @param array{browser_download_url: string, name: string} $asset
                 * @return array<string, string>
                 */
                static function (array $downloadMap, array $asset) use ($platforms): array {
                    if (!str_ends_with($asset['name'], '.tar.gz') && !str_ends_with($asset['name'], '.zip')) {
                        return $downloadMap;
                    }

                    if ($platforms &&
                        preg_match(
                            '/(' . implode('|', array_map(preg_quote(...), $platforms)) . ')/',
                            $asset['name'],
                        ) ===
                        0) {
                        return $downloadMap;
                    }

                    $downloadMap[$asset['name']] = $asset['browser_download_url'];

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
     * @param Composer $composer
     * @return array{platforms: list<string>}
     */
    private function getMagoConfig(Composer $composer): array
    {
        $extra = $composer->getPackage()->getExtra();

        return [
            'platforms' => $extra['mago']['platforms'] ?? [],
        ];
    }
}
