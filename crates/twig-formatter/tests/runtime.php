<?php

declare(strict_types=1);

namespace Mago\Tests\TwigFormatter;

use Generator;
use RuntimeException;
use Twig\Environment;
use Twig\Loader\ArrayLoader;
use Twig\TwigFunction;
use Twig\TwigTest;

require dirname(__DIR__, 3) . '/vendor/autoload.php';
require __DIR__ . '/runtime/tokens.php';
require __DIR__ . '/runtime/raw.php';

/** @return Generator<string, string> */
function cases(): Generator
{
    foreach (glob(__DIR__ . '/runtime/cases/*.twig') as $path) {
        $original = file_get_contents($path);
        $name = basename($path);
        foreach (["\n", "\r\n", "\r"] as $index => $newline) {
            $source = str_replace("\n", $newline, $original);
            if ($name !== 'whitespace_control.twig') {
                yield "$name:$index" => $source;
                yield "$name:$index:no-final-newline" => rtrim($source, "\r\n");
                if ($name !== 'inheritance.twig') {
                    yield "$name:$index:bom" => "\xEF\xBB\xBF" . $source;
                    yield "$name:$index:nul" => "\0" . $source . "\0";
                }
                continue;
            }

            foreach (['-', '~', ''] as $left) {
                foreach (['-', '~', ''] as $right) {
                    foreach (['', ' ', "\t", $newline] as $body) {
                        yield "$name:$index:$left:$right:" . bin2hex($body) => strtr($source, [
                            '__LEFT__' => $left,
                            '__RIGHT__' => $right,
                            '__BODY__' => $body,
                        ]);
                    }
                }
            }
        }
    }
}

function write(string $path, string $contents): void
{
    if (file_put_contents($path, $contents) !== strlen($contents)) {
        throw new RuntimeException("Could not write $path.");
    }
}

function format(string $binary, string $config, string $source): string
{
    $directory = dirname($config);
    write($directory . '/case.twig', $source);
    $process = proc_open(
        [
            $binary,
            '--no-version-check',
            '--threads',
            '1',
            '--workspace',
            $directory,
            '--config',
            $config,
            'twig:fmt',
            '--stdin-input',
            '--stdin-filepath',
            'case.twig',
        ],
        [['file', $directory . '/case.twig', 'r'], ['pipe', 'w'], ['file', $directory . '/stderr.log', 'w']],
        $pipes,
    );
    if ($process === false) {
        throw new RuntimeException('Could not start Mago.');
    }

    $output = stream_get_contents($pipes[1]);
    fclose($pipes[1]);
    $status = proc_close($process);
    if ($status !== 0 || $output === false) {
        $error = file_get_contents($directory . '/stderr.log');
        throw new RuntimeException("Formatting failed: $error\n$source");
    }

    return $output;
}

/** @param list<string> $trace */
function environment(ArrayLoader $loader, array &$trace, bool $autoescape): Environment
{
    $twig = new Environment($loader, ['autoescape' => $autoescape ? 'html' : false, 'strict_variables' => true]);
    $twig->addTokenParser(new RawTokenParser());
    $twig->addFunction(new TwigFunction('record', static function (string $label, mixed $value = null) use (
        &$trace,
    ): mixed {
        $trace[] = $label;
        return $value ?? $label;
    }));
    $twig->addFunction(new TwigFunction('f', static fn(string $value, int $n): string => "$value:$n"));
    $twig->addFunction(new TwigFunction('g', static fn(int $in, int $not): int => $in + $not));
    $twig->addTest(new TwigTest('s is', static fn(mixed $value): bool => false));
    $twig->addTest(new TwigTest('s not', static fn(mixed $value): bool => false));
    $twig->addTest(new TwigTest('not', static fn(mixed $value): bool => false));
    foreach (['starts with', 'ends with', 'has some', 'has every'] as $name) {
        $twig->addTest(new TwigTest($name, static fn(mixed $value): bool => true));
    }
    foreach (['foo has', 'foo starts', 'foo ends'] as $name) {
        $twig->addTest(new TwigTest(
            $name,
            static fn(mixed $value, mixed $other): bool => $value !== $other,
            ['one_mandatory_argument' => true],
        ));
    }
    return $twig;
}

/**
 * @param array<string, mixed> $context
 * @param list<string> $trace
 * @return array{string, list<string>}
 */
function render(Environment $twig, ArrayLoader $loader, string $source, array $context, array &$trace): array
{
    $loader->setTemplate('case.twig', $source);
    $trace = [];
    $output = $twig->render('case.twig', $context);
    return [$output, $trace];
}

$binary = $argv[1] ?? dirname(__DIR__, 3) . '/target/debug/mago' . (PHP_OS_FAMILY === 'Windows' ? '.exe' : '');
$templates = [];
foreach (glob(__DIR__ . '/runtime/templates/*.twig') as $path) {
    $templates[basename($path)] = file_get_contents($path);
}

$loader = new ArrayLoader($templates);
$trace = [];
$environments = [environment($loader, $trace, false), environment($loader, $trace, true)];
$context = [
    'x' => 'hello <world>',
    'y' => 'test',
    'a' => true,
    'b' => 'yes',
    'c' => 'no',
    'not' => 2,
    'list' => [1, 2, 3, 4],
    'obj' => ['child' => 'ok', 'alphabeta' => 'dynamic', 'a' => 'A', 'b' => 'B'],
    'key' => 'child',
    'with' => 'with',
    'some' => 'some',
    'every' => 'every',
];
$contexts = [
    $context + ['flag' => true],
    array_replace($context, ['flag' => false, 'a' => false, 'y' => '', 'list' => []]),
];
$profiles = [
    'default' => '',
    'narrow' => 'print-width = 30',
    'tabs' => "print-width = 30\nuse-tabs = true\ntab-width = 8",
    'crlf' => "print-width = 30\nend-of-line = \"crlf\"",
    'cr' => "print-width = 30\nend-of-line = \"cr\"",
    'tiny' => "print-width = 1\nuse-tabs = true\ntab-width = 0",
];
$directory = sys_get_temp_dir() . '/mago-twig-runtime-' . bin2hex(random_bytes(8));
if (!mkdir($directory)) {
    throw new RuntimeException('Could not create the test directory.');
}

$config = $directory . '/mago.toml';
$checked = 0;
try {
    foreach ($profiles as $profile => $settings) {
        write($config, "[source]\npaths = []\n[twig.formatter]\n$settings\n");
        foreach (cases() as $name => $source) {
            $formatted = format($binary, $config, $source);
            foreach ($environments as $twig) {
                if (tokens($twig, $source) !== tokens($twig, $formatted)) {
                    throw new RuntimeException("Twig tokens changed: $name ($profile)\n$formatted");
                }

                foreach ($contexts as $context) {
                    $expected = render($twig, $loader, $source, $context, $trace);
                    $actual = render($twig, $loader, $formatted, $context, $trace);
                    if ($actual !== $expected) {
                        throw new RuntimeException("Rendering or call order changed: $name ($profile)\n$formatted");
                    }
                    ++$checked;
                }
            }

            if (format($binary, $config, $formatted) !== $formatted) {
                throw new RuntimeException("Unstable formatting: $name ($profile)\n$formatted");
            }
        }
    }
} finally {
    foreach ([$config, $directory . '/case.twig', $directory . '/stderr.log'] as $path) {
        if (is_file($path)) {
            unlink($path);
        }
    }
    rmdir($directory);
}

printf("Twig %s: %d byte-exact rendering and call-order checks passed.\n", Environment::VERSION, $checked);
