<?php

declare(strict_types=1);

namespace Mago\Tests\TwigLinter;

use RuntimeException;
use Twig\Environment;

require $argv[2] ?? dirname(__DIR__, 3) . '/vendor/autoload.php';
require __DIR__ . '/runtime/support.php';

$binary = $argv[1] ?? dirname(__DIR__, 3) . '/target/debug/mago' . (PHP_OS_FAMILY === 'Windows' ? '.exe' : '');
$rules = 'twig-named-argument-separator,twig-single-quotes,twig-end-block-name';
$directory = sys_get_temp_dir() . '/mago-twig-lint-runtime-' . bin2hex(random_bytes(8));
if (!mkdir($directory)) {
    throw new RuntimeException('Could not create the test directory.');
}
$trace = [];
$environments = [environment(false, $trace), environment(true, $trace)];
$checked = 0;
try {
    $checked += checkArguments($environments);
    foreach (glob(__DIR__ . '/runtime/cases/*.twig') as $path) {
        $original = file_get_contents($path);
        foreach (["\n", "\r\n", "\r"] as $newline) {
            foreach (['', '-', '~'] as $left) {
                foreach (['', '-', '~'] as $right) {
                    if (!str_starts_with(basename($path), 'trim') && ($left !== '' || $right !== '')) {
                        continue;
                    }
                    $source = str_replace("\n", $newline, strtr($original, ['__LEFT__' => $left, '__RIGHT__' => $right]));
                    file_put_contents($directory . '/case.twig', $source);
                    file_put_contents($directory . '/mago.toml', "[source]\npaths=[]\n[twig.linter]\nversion='3.30'\n");
                    run($binary, $directory, ['twig:lint', 'case.twig', '--only', $rules, '--fix']);
                    $fixed = file_get_contents($directory . '/case.twig');
                    if (basename($path) === 'comments.twig'
                        && (!str_contains($fixed, '"inline"') || !str_contains($fixed, '"plain"'))) {
                        throw new RuntimeException('The counted inline pragma did not keep its quoted arguments.');
                    }
                    run($binary, $directory, ['twig:lint', 'case.twig', '--only', $rules, '--minimum-fail-level', 'note']);
                    foreach ($environments as [$twig, $loader]) {
                        if (tokens($twig, $source) !== tokens($twig, $fixed)) {
                            throw new RuntimeException("Twig tokens changed outside allowed separators/end names: $path\n$fixed");
                        }
                        foreach ([['value' => '<hello>'], ['value' => '']] as $context) {
                            if (render($twig, $loader, $source, $trace, $context) !== render($twig, $loader, $fixed, $trace, $context)) {
                                throw new RuntimeException("Rendered bytes or call order changed: $path\n$fixed");
                            }
                            ++$checked;
                        }
                    }
                    run($binary, $directory, ['fmt', 'case.twig']);
                    $formatted = file_get_contents($directory . '/case.twig');
                    run($binary, $directory, ['twig:lint', 'case.twig', '--only', $rules, '--fix', '--fail-on-remaining']);
                    run($binary, $directory, ['fmt', 'case.twig', '--check']);
                    if (file_get_contents($directory . '/case.twig') !== $formatted) {
                        throw new RuntimeException("Lint and format did not settle: $path");
                    }
                }
            }
        }
    }
    $source = file_get_contents(__DIR__ . '/fixtures/raw_boundaries.twig');
    file_put_contents($directory . '/case.twig', $source);
    foreach ([[], ['--potentially-unsafe'], ['--unsafe']] as $flags) {
        run($binary, $directory, ['twig:lint', 'case.twig', '--only', 'twig-no-raw', '--fix', ...$flags]);
        $fixed = file_get_contents($directory . '/case.twig');
        if ($fixed !== $source) {
            throw new RuntimeException('The raw policy must not change the source at any safety level.');
        }
        foreach ($environments as [$twig, $loader]) {
            $context = ['value' => '<hello>', 'b' => 10, 'and' => 3];
            if (tokens($twig, $source) !== tokens($twig, $fixed)
                || render($twig, $loader, $source, $trace, $context) !== render($twig, $loader, $fixed, $trace, $context)) {
                throw new RuntimeException('The raw policy changed tokens, rendered bytes, or call order.');
            }
            ++$checked;
        }
    }
} finally {
    foreach (glob($directory . '/*') as $path) {
        unlink($path);
    }
    rmdir($directory);
}
printf("Twig %s: %d normalized token and exact render/call-order checks passed.\n", Environment::VERSION, $checked);
