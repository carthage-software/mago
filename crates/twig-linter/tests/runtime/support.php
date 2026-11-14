<?php

declare(strict_types=1);

namespace Mago\Tests\TwigLinter;

use RuntimeException;
use Twig\Environment;
use Twig\Loader\ArrayLoader;
use Twig\Source;
use Twig\Token;
use Twig\TwigFunction;
use Twig\TwigFilter;
use Twig\TwigTest;

function run(string $binary, string $directory, array $arguments): void
{
    $process = proc_open(
        [$binary, '--no-version-check', '--threads', '1', '--colors', 'never', '--workspace', $directory, ...$arguments],
        [['file', PHP_OS_FAMILY === 'Windows' ? 'NUL' : '/dev/null', 'r'], ['pipe', 'w'], ['pipe', 'w']],
        $pipes,
        $directory,
    );
    if ($process === false) {
        throw new RuntimeException('Could not start Mago.');
    }
    stream_set_blocking($pipes[1], false);
    stream_set_blocking($pipes[2], false);
    $stdout = $stderr = '';
    $deadline = microtime(true) + 15;
    do {
        $stdout .= stream_get_contents($pipes[1]);
        $stderr .= stream_get_contents($pipes[2]);
        $status = proc_get_status($process);
        if (microtime(true) > $deadline || strlen($stdout) + strlen($stderr) > 1_048_576) {
            proc_terminate($process, 9);
            fclose($pipes[1]);
            fclose($pipes[2]);
            proc_close($process);
            throw new RuntimeException('Mago exceeded the runtime test time or output limit.');
        }
        if ($status['running']) {
            usleep(10_000);
        }
    } while ($status['running']);
    $stdout .= stream_get_contents($pipes[1]);
    $stderr .= stream_get_contents($pipes[2]);
    fclose($pipes[1]);
    fclose($pipes[2]);
    $exit = proc_close($process);
    if (($status['exitcode'] >= 0 ? $status['exitcode'] : $exit) !== 0) {
        $source = file_get_contents($directory . '/case.twig');
        throw new RuntimeException('Mago failed (' . implode(' ', $arguments) . "): $stdout\n$stderr\n$source");
    }
}

function tokens(Environment $twig, string $source): array
{
    $stream = $twig->tokenize(new Source($source, 'case'));
    $tokens = [];
    $block = false;
    $closing = false;
    while (!$stream->isEOF()) {
        $token = $stream->next();
        $type = $token->getType();
        $value = $token->getValue();
        if ($type === Token::BLOCK_START_TYPE) {
            $block = true;
        } elseif ($type === Token::BLOCK_END_TYPE) {
            $block = false;
            $closing = false;
        } elseif ($block && $closing && $type === Token::NAME_TYPE) {
            $closing = false;
            continue;
        } elseif ($block && $type === Token::NAME_TYPE && in_array($value, ['endblock', 'endmacro'], true)) {
            $closing = true;
        }
        if ($type === Token::OPERATOR_TYPE && $value === '=') {
            [$type, $value] = [Token::PUNCTUATION_TYPE, ':'];
        }
        $tokens[] = [$type, $value];
    }
    return $tokens;
}

function environment(bool $autoescape, array &$trace): array
{
    $loader = new ArrayLoader([
        'fragment' => "<i> {{ value }} </i>\n",
        'base' => 'Base:{% block body %}base{% endblock %}',
        'macros' => '{% macro echo(value) %}{{ value }}{% endmacro %}',
        'arguments' => '{% macro exact(foo_bar, foobar) %}{{ foo_bar }}:{{ foobar }}{% endmacro %}',
    ]);
    $twig = new Environment($loader, ['strict_variables' => true, 'autoescape' => $autoescape ? 'html' : false]);
    $twig->addFunction(new TwigFunction('record', static function (string $label, mixed $value) use (&$trace): mixed {
        $trace[] = [$label, $value];
        return $value;
    }));
    $twig->addFunction(new TwigFunction('argument_probe', static fn (mixed $fooBar): mixed => $fooBar));
    $twig->addFilter(new TwigFilter('argument_probe', static fn (mixed $value, mixed $fooBar): mixed => $fooBar));
    $twig->addTest(new TwigTest('argument_probe', static fn (mixed $value, bool $fooBar): bool => $fooBar));
    return [$twig, $loader];
}

function checkArguments(array $environments): int
{
    $cases = [
        '{{ argument_probe(foo_bar: 1, foobar: 2) }}' => '2',
        '{{ argument_probe(fooBar: 1, foo_bar: 2, FOO_BAR: 3) }}' => '3',
        '{{ 0|argument_probe(foo_bar: 1, foobar: 2) }}' => '2',
        "{{ 0 is argument_probe(foo_bar: false, foobar: true) ? 'yes' : 'no' }}" => 'yes',
        "{% from 'arguments' import exact %}{{ exact(foo_bar: 1, foobar: 2) }}" => '1:2',
    ];
    foreach ($environments as [$twig]) {
        foreach ($cases as $source => $expected) {
            if ($twig->createTemplate($source)->render() !== $expected) {
                throw new RuntimeException('Twig callable/macro named argument normalization changed.');
            }
        }
    }
    return count($cases) * count($environments);
}

function render(Environment $twig, ArrayLoader $loader, string $source, array &$trace, array $context): array
{
    $loader->setTemplate('case', $source);
    $trace = [];
    $output = $twig->render('case', $context);
    return [$output, $trace];
}
