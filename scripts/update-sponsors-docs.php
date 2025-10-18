#!/usr/bin/env php
<?php

declare(strict_types=1);

namespace Mago\Scripts;

use RuntimeException;

use function array_slice;
use function explode;
use function file_get_contents;
use function file_put_contents;
use function implode;
use function json_decode;
use function krsort;
use function preg_replace;
use function sprintf;
use function str_contains;
use function str_starts_with;

use const JSON_THROW_ON_ERROR;

const SPONSORS_URL = 'https://raw.githubusercontent.com/azjezz/azjezz/develop/sponsors.json';

const DOCS_INDEX_PATH = __DIR__ . '/../docs/index.md';

const README_PATH = __DIR__ . '/../README.md';

const SPONSORS_PATH = __DIR__ . '/../SPONSORS.md';

/**
 * @mago-expect lint:excessive-parameter-list
 */
final class Sponsor
{
    public function __construct(
        public readonly string $login,
        public readonly string $name,
        public readonly string $avatarUrl,
        public readonly null|string $websiteUrl,
        public readonly int $monthlyPriceInDollars,
        public readonly bool $isCustomAmount,
        public readonly bool $isOneTime,
    ) {}
}

/**
 * @mago-expect lint:cyclomatic-complexity
 * @mago-expect lint:kan-defect
 */
final class SponsorsData
{
    private const LARGE_SPONSOR_THRESHOLD = 100;
    private const MEDIUM_SPONSOR_THRESHOLD = 50;
    private const SMALL_SPONSOR_THRESHOLD = 25;

    /**
     * @var array<non-negative-int, list<Sponsor>>
     */
    public readonly array $sponsorsByAmount;

    /**
     * @param array<non-negative-int, list<Sponsor>> $sponsorsByAmount
     */
    public function __construct(array $sponsorsByAmount)
    {
        krsort($sponsorsByAmount);

        $this->sponsorsByAmount = $sponsorsByAmount;
    }

    public static function fetch(): SponsorsData
    {
        $sponsors_json = file_get_contents(SPONSORS_URL);
        if (false === $sponsors_json) {
            throw new RuntimeException('Failed to fetch sponsors data.');
        }

        /**
         * @var array<
         *  int<0, max>,
         *  list<array{login: string, name: string, avatarUrl: string, websiteUrl: ?string, monthlyPriceInDollars: int, isCustomAmount: bool, isOneTime: bool}>
         * > $sponsors_by_tier
         */
        $sponsors_by_tier = json_decode($sponsors_json, true, 512, JSON_THROW_ON_ERROR);

        $sponsorsByAmount = [];
        foreach ($sponsors_by_tier as $monthlyDollar => $sponsors) {
            $amountSponsors = $sponsorsByAmount[$monthlyDollar] ?? [];
            foreach ($sponsors as $sponsor) {
                $sponsor = new Sponsor(
                    login: $sponsor['login'],
                    name: $sponsor['name'],
                    /** @mago-expect lint:no-shorthand-ternary */
                    avatarUrl: preg_replace('/\?s=\d+$/', '', $sponsor['avatarUrl']) ?: $sponsor['avatarUrl'],
                    websiteUrl: $sponsor['websiteUrl'] ?? null,
                    monthlyPriceInDollars: $sponsor['monthlyPriceInDollars'],
                    isCustomAmount: $sponsor['isCustomAmount'],
                    isOneTime: $sponsor['isOneTime'],
                );

                $amountSponsors[] = $sponsor;
            }

            $sponsorsByAmount[$monthlyDollar] = $amountSponsors;
        }

        return new static(sponsorsByAmount: $sponsorsByAmount);
    }

    public function renderForDocs(): string
    {
        $large_sponsors_html = '';
        $medium_sponsors_html = '';
        $small_sponsors_html = '';

        foreach ($this->sponsorsByAmount as $amount => $sponsors) {
            foreach ($sponsors as $sponsor) {
                $url = $sponsor->websiteUrl ?? sprintf('https://github.com/%s', $sponsor->login);
                if (!str_starts_with($url, 'http://') && !str_starts_with($url, 'https://')) {
                    $url = sprintf('http://%s', $url); // Corrected from https to http
                }

                if ($amount >= self::LARGE_SPONSOR_THRESHOLD) {
                    $large_sponsors_html .= sprintf(
                        '<div class="sponsor-item-large"><a href="%s" title="%s" target="_blank" rel="noopener"><img src="%s&s=256" alt="%s"></a><div class="sponsor-name"><a href="%s" target="_blank" rel="noopener">%s</a></div></div>'
                        . "\n",
                        $url,
                        $sponsor->name,
                        $sponsor->avatarUrl,
                        $sponsor->name,
                        $url,
                        $sponsor->name,
                    );

                    continue;
                }

                if ($amount >= self::MEDIUM_SPONSOR_THRESHOLD) {
                    $medium_sponsors_html .= sprintf(
                        '<div class="sponsor-item-medium"><a href="%s" title="%s" target="_blank" rel="noopener"><img src="%s&s=96" alt="%s"></a></div>'
                        . "\n",
                        $url,
                        $sponsor->name,
                        $sponsor->avatarUrl,
                        $sponsor->name,
                    );

                    continue;
                }

                if ($amount < self::SMALL_SPONSOR_THRESHOLD) {
                    continue;
                }

                $small_sponsors_html .= sprintf(
                    '<a class="sponsor-item-small" href="%s" title="%s" target="_blank" rel="noopener"><img src="%s&s=48" alt="%s"></a>'
                    . "\n",
                    $url,
                    $sponsor->name,
                    $sponsor->avatarUrl,
                    $sponsor->name,
                );
            }
        }

        $html = '<div class="sponsors-list">';
        if ('' !== $large_sponsors_html) {
            $html .= '<div class="sponsors-list-large">' . $large_sponsors_html . '</div>';
        }

        if ('' !== $medium_sponsors_html) {
            if ('' !== $large_sponsors_html) {
                $html .= '<hr>';
            } // Corrected spacing
            $html .= '<div class="sponsors-list-medium">' . $medium_sponsors_html . '</div>';
        }

        if ('' !== $small_sponsors_html) {
            if ('' !== $large_sponsors_html || '' !== $medium_sponsors_html) {
                $html .= '<hr>';
            } // Corrected spacing
            $html .= '<div class="sponsors-list-small">' . $small_sponsors_html . '</div>';
        }

        $html .= '</div>';
        $html .= '<hr />';
        $html .= '<p style="text-align: center;">Your logo here? <a href="https://github.com/sponsors/azjezz" target="_blank" rel="noopener">Become a sponsor!</a></p>';

        return $html;
    }

    public function renderForReadme(): string
    {
        $large_sponsors_html = '';
        $medium_sponsors_html = '';
        $small_sponsors_html = '';

        foreach ($this->sponsorsByAmount as $amount => $sponsors) {
            if ($amount < self::SMALL_SPONSOR_THRESHOLD) {
                continue;
            }

            foreach ($sponsors as $sponsor) {
                $url = $sponsor->websiteUrl ?? sprintf('https://github.com/%s', $sponsor->login);
                if (!str_starts_with($url, 'http://') && !str_starts_with($url, 'https://')) {
                    $url = sprintf('http://%s', $url); // Corrected from https to http
                }

                if ($amount >= self::LARGE_SPONSOR_THRESHOLD) {
                    $large_sponsors_html .= sprintf(
                        '<a href="%s" title="%s"><kbd><img src="%s&s=240" width="120" height="120" alt="%s" /></kbd></a>',
                        $url,
                        $sponsor->name,
                        $sponsor->avatarUrl,
                        $sponsor->name,
                    );

                    continue;
                }

                if ($amount >= self::MEDIUM_SPONSOR_THRESHOLD) {
                    $medium_sponsors_html .= sprintf(
                        '<a href="%s" title="%s"><kbd><img src="%s&s=160" width="80" height="80" alt="%s" /></kbd></a>',
                        $url,
                        $sponsor->name,
                        $sponsor->avatarUrl,
                        $sponsor->name,
                    );

                    continue;
                }

                $small_sponsors_html .= sprintf(
                    '<a href="%s" title="%s"><kbd><img src="%s&s=96" width="48" height="48" alt="%s" /></kbd></a>',
                    $url,
                    $sponsor->name,
                    $sponsor->avatarUrl,
                    $sponsor->name,
                );
            }
        }

        $html = '';
        if ('' !== $large_sponsors_html) {
            $html .= '<p align="center">' . $large_sponsors_html . '</p>';
        }
        if ('' !== $medium_sponsors_html) {
            $html .= '<p align="center">' . $medium_sponsors_html . '</p>';
        }
        if ('' !== $small_sponsors_html) {
            $html .= '<p align="center">' . $small_sponsors_html . '</p>';
        }

        if ('' !== $html) {
            $html .= "\n\n";
            $html .= '[See all sponsors](SPONSORS.md)';
        }

        return $html;
    }

    public function renderForSponsorsPage(): string
    {
        $tiers = [];
        foreach ($this->sponsorsByAmount as $sponsors) {
            $tier_html = '';
            foreach ($sponsors as $sponsor) {
                $url = $sponsor->websiteUrl ?? sprintf('https://github.com/%s', $sponsor->login);
                if (!str_starts_with($url, 'http://') && !str_starts_with($url, 'https://')) {
                    $url = sprintf('http://%s', $url); // Corrected from https to http
                }

                $tier_html .= sprintf(
                    '<a href="%s" title="%s"><kbd><img src="%s&s=160" width="80" height="80" alt="%s" /></kbd></a>',
                    $url,
                    $sponsor->name,
                    $sponsor->avatarUrl,
                    $sponsor->name,
                );
            }

            $tiers[] = $tier_html;
        }

        return implode("\n\n---\n\n", $tiers);
    }
}

function overwrite_sponsors_file(string $new_content): void
{
    $header = <<<MD
    # Sponsors

    A heartfelt thank you to the generous individuals and organizations listed below. Their support is instrumental in the continued development and maintenance of [Psl](https://github.com/azjezz/psl) and [Mago](https://github.com/carthage-software/mago).

    To become a sponsor, please visit [the sponsorship page](https://github.com/sponsors/azjezz).

    ---
    MD;

    $full_content = $header . "\n" . $new_content;
    file_put_contents(SPONSORS_PATH, $full_content);
}

function update_markdown_file(string $filePath, string $start_marker, string $end_marker, string $new_content): void
{
    $file_content = file_get_contents($filePath);
    if (false === $file_content) {
        throw new RuntimeException('Failed to read {' . $filePath . '}');
    }

    if (!str_contains($file_content, $start_marker) || !str_contains($file_content, $end_marker)) {
        throw new RuntimeException('Markers not found in {' . $filePath . '}');
    }

    /** @var list{string, string} $parts */
    $parts = explode($start_marker, $file_content);

    $head = $parts[0];
    /** @var list{string, string} $tail_parts */
    $tail_parts = explode($end_marker, $parts[1]);

    $tail = implode($end_marker, array_slice($tail_parts, 1));

    $updated_content = $head . $start_marker . "\n" . $new_content . "\n" . $end_marker . $tail;

    file_put_contents($filePath, $updated_content);
}

try {
    $sponsorsData = SponsorsData::fetch();

    // Update docs/index.md
    namespace\update_markdown_file(
        DOCS_INDEX_PATH,
        '<!-- SPONSORS_START -->',
        '<!-- SPONSORS_END -->',
        $sponsorsData->renderForDocs(),
    );

    echo "✅ Sponsors section in docs/index.md updated successfully.\n";

    // Update README.md
    namespace\update_markdown_file(
        README_PATH,
        '<!-- START-SPONSORS -->',
        '<!-- END-SPONSORS -->',
        $sponsorsData->renderForReadme(),
    );

    echo "✅ Sponsors section in README.md updated successfully.\n";

    // Update SPONSORS.md
    namespace\overwrite_sponsors_file($sponsorsData->renderForSponsorsPage());

    echo "✅ SPONSORS.md updated successfully.\n";
} catch (RuntimeException $e) {
    echo '❌ Error: {' . $e->getMessage() . "}\n";

    exit(1);
}
