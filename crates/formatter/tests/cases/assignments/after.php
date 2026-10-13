<?php

$a = 1;

class A
{
    public const CONSTANT = 2;
}

const GLOBAL_CONSTANT = 3;

enum Status: int
{
    case Active = 1;
}

class B
{
    public int $property = 4;
}

$arr = [
    'key1' => 'value1',
    'key2' => 'value2',
];

$longestCommandName = max(
    arr($commands)
        ->flatMap(fn(array $group) => $group)
        ->map(fn(ConsoleCommand $command) => mb_strlen($command->getName()))
        ->toArray(),
) + 4;

$content .=
    PHP_EOL
    . arr($validationErrors)
        ->map(fn(string $error) => "  <style=\"fg-yellow\">{$error}</style>")
        ->implode(PHP_EOL)
        ->append(PHP_EOL)
        ->toString();

$content .= arr($validationErrors)
    ->map(fn(string $error) => "  <style=\"fg-yellow\">{$error}</style>")
    ->implode(PHP_EOL)
    ->append(PHP_EOL)
    ->toString() . PHP_EOL;

$content .= arr($validationErrors)
    ->map(fn(string $error) => "  <style=\"fg-yellow\">{$error}</style>")
    ->implode(PHP_EOL)
    ->append(PHP_EOL)
    ->toString();

$answerAsString = arr($answers)
    ->map(fn(Option $option) => $option->displayValue)
    ->join(', ', ' and ')
    ->trim()
    ->toString() ?: 'no option';

function foo()
{
    if ($write) {
        $writable =
            str_contains($meta['mode'], 'x')
            || str_contains($meta['mode'], 'w')
            || str_contains($meta['mode'], 'c')
            || str_contains($meta['mode'], 'a')
            || str_contains($meta['mode'], '+');
    }

    $target_directory = Env\temp_dir() . DIRECTORY_SEPARATOR . 'you-shall-not-pass';

    if (($i + 1) < $binary_length) {
        $byte1 = $chunk[2];
        $dest .=
            static::encode6Bits($byte0 >> 2)
            . static::encode6Bits((($byte0 << 4) | ($byte1 >> 4)) & 63)
            . static::encode6Bits(($byte1 << 2) & 63);
    }

    $target_file =
        $target_directory . DIRECTORY_SEPARATOR . 'fails-on-subdir-creation' . DIRECTORY_SEPARATOR . 'somefile.txt';

    /** @var PackageManager */
    $packageManager = PackageManager::detect($cwd) ?? $this->console->ask(
        question: 'Which package manager do you wish to use?',
        options: PackageManager::class,
        default: PackageManager::BUN,
        validation: [
            new Enum(PackageManager::class),
        ],
    );
}
