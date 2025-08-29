#!/usr/bin/env php
<?php

/**
 * A PHP script to generate markdown documentation for linter rules from the Mago CLI tool.
 *
 * It fetches rule data and configuration, organizes rules by category and integration,
 * and creates markdown files in the specified documentation directory.
 */

declare(strict_types=1);

/**
 * This enum must stay in sync with the `Integration` enum in `crates/linter/src/integration.rs`.
 */
enum Integration: int
{
    case Psl = 0;
    case Guzzle = 1;
    case Monolog = 2;
    case Carbon = 3;
    case Amphp = 4;
    case ReactPHP = 5;
    case Symfony = 6;
    case Laravel = 7;
    case Tempest = 8;
    case Neutomic = 9;
    case Spiral = 10;
    case CakePHP = 11;
    case Yii = 12;
    case Laminas = 13;
    case Cycle = 14;
    case Doctrine = 15;
    case WordPress = 16;
    case Drupal = 17;
    case Magento = 18;
    case PHPUnit = 19;
    case Pest = 20;
    case Behat = 21;
    case Codeception = 22;
    case PHPSpec = 23;
}

try {
    main();
} catch (Exception $e) {
    writeln('❌', 'An unexpected error occurred: %s', $e->getMessage());
    exit(1);
}

/**
 * The main entry point for the script.
 *
 * @mago-expect lint:no-boolean-literal-comparison
 */
function main(): void
{
    $script_dir = dirname(__FILE__);
    $project_root = realpath($script_dir . '/..');
    if ($project_root === false) {
        throw new RuntimeException('Failed to determine project root directory.');
    }

    $docs_dir = $project_root . '/docs';
    $rules_target_dir = $docs_dir . '/tools/linter/rules';
    $mago_executable = $project_root . '/target/release/mago';

    writeln('✨', 'Starting linter rule documentation generation...');

    // Build the Mago executable first to ensure it's up-to-date.
    writeln('🏗️ ', 'Building Mago executable in release mode...');
    $build_result = -1;
    passthru('cargo build --release', $build_result);
    if ($build_result !== 0) {
        throw new RuntimeException('Failed to build Mago executable.');
    }
    writeln('✅', 'Mago executable built successfully.');

    $rules = fetch_rules_from_mago($mago_executable);
    $linter_config = fetch_linter_config($mago_executable);
    clean_and_prepare_directories($rules_target_dir);
    [$rules_by_category, $rules_by_integration] = group_rules($rules);

    generate_category_files($rules_by_category, $rules_target_dir, $linter_config);
    generate_overview_page($docs_dir, $rules_by_category, $rules_by_integration);

    writeln('✅', 'All documentation files have been generated successfully.');
}

/**
 * Fetches the list of all linter rules from the Mago CLI.
 *
 * @return list<array{
 *  name: string,
 *  code: string,
 *  description: string,
 *  good_example: string,
 *  bad_example: string,
 *  category: string,
 *  requires: int,
 *  php: array{min: ?string, max: ?string}
 * }>
 *
 * @mago-expect lint:no-boolean-literal-comparison
 */
function fetch_rules_from_mago(string $mago_executable): array
{
    writeln('🔍', 'Fetching rule data from Mago...');
    $command = "{$mago_executable} lint --pedantic --list-rules --json";
    $json_output = shell_exec($command);

    if ($json_output === null || $json_output === false) {
        throw new RuntimeException('Failed to execute Mago command to get rules.');
    }

    /**
     * @var list<array{
     *   name: string,
     *   code: string,
     *   description: string,
     *   good_example: string,
     *   bad_example: string,
     *   category: string,
     *   requires: int,
     *   php: array{min: ?string, max: ?string}
     * }>
     */
    $rules = json_decode($json_output, true, 512, JSON_THROW_ON_ERROR);

    writeln('✅', 'Fetched %d rules successfully.', count($rules));

    return $rules;
}

/**
 * Fetches the default linter configuration from the Mago CLI.
 *
 * @return array{rules: array<string, array{'enabled': bool, 'level': string, ...<string, scalar|array<scalar>>}>}
 *
 * @mago-expect lint:no-boolean-literal-comparison
 */
function fetch_linter_config(string $mago_executable): array
{
    writeln('⚙️ ', 'Fetching default linter configuration...');
    $command = "{$mago_executable} config --show linter";
    $json_output = shell_exec($command);

    if ($json_output === null || $json_output === false) {
        throw new RuntimeException('Failed to execute Mago command to get config.');
    }

    /** @var array{rules: array<string, array{'enabled': bool, 'level': string, ...<string, scalar>}>} */
    return json_decode($json_output, true, 512, JSON_THROW_ON_ERROR);
}

/**
 * Cleans the target directory and prepares it for new content.
 */
function clean_and_prepare_directories(string $rules_target_dir): void
{
    writeln('🧹', 'Cleaning target directory: %s', $rules_target_dir);

    if (is_dir($rules_target_dir)) {
        delete_directory($rules_target_dir);
    }

    mkdir($rules_target_dir, 0o755, true);

    writeln('✅', 'Cleaned and re-created directory: %s', $rules_target_dir);
}

/**
 * Groups rules by category and by integration.
 *
 * @param list<array{
 *   name: string,
 *   code: string,
 *   description: string,
 *   good_example: string,
 *   bad_example: string,
 *   category: string,
 *   requires: int,
 *   php: array{min: ?string, max: ?string}
 * }> $rules
 *
 * @return list{
 *  array<string, array{
 *    kebab: string,
 *    rules: list<array{
 *      name: string,
 *      code: string,
 *      description: string,
 *      good_example: string,
 *      bad_example: string,
 *      category: string,
 *      requires: int,
 *      php: array{min: ?string, max: ?string}
 *    }>
 *  }>,
 *  array<string, list<array{
 *    name: string,
 *    code: string,
 *    description: string,
 *    good_example: string,
 *    bad_example: string,
 *    category: string,
 *    requires: int,
 *    php: array{min: ?string, max: ?string}
 *  }>>
 * }
 */
function group_rules(array $rules): array
{
    /**
     * @var array<string, array{kebab: string, rules: list<array{
     *   name: string,
     *   code: string,
     *   description: string,
     *   good_example: string,
     *   bad_example: string,
     *   category: string,
     *   requires: int,
     *   php: array{min: ?string, max: ?string}
     * }>}>
     */
    $rules_by_category = [];

    /**
     * @var array<string, list<array{
     *   name: string,
     *   code: string,
     *   description: string,
     *   good_example: string,
     *   bad_example: string,
     *   category: string,
     *   requires: int,
     *   php: array{min: ?string, max: ?string}
     * }>>
     */
    $rules_by_integration = [];

    foreach ($rules as $rule) {
        $categoryKebab = to_kebab_case($rule['category']);
        $rules_by_category[$rule['category']]['kebab'] = $categoryKebab;
        $rules_by_category[$rule['category']]['rules'][] = $rule;

        if ($rule['requires'] > 0) {
            foreach (Integration::cases() as $case) {
                if (($rule['requires'] >> $case->value) & 1) {
                    $rules_by_integration[$case->name][] = $rule;
                }
            }
        }
    }

    ksort($rules_by_category);
    ksort($rules_by_integration);

    return [$rules_by_category, $rules_by_integration];
}

/**
 * Generates a single markdown file for each rule category.
 *
 * @param array<string, array{kebab: string, rules: list<array{
 *   name: string,
 *   code: string,
 *   description: string,
 *   good_example: string,
 *   bad_example: string,
 *   category: string,
 *   requires: int,
 *   php: array{min: ?string, max: ?string}
 * }>}> $rules_by_category
 *
 * @param array{rules: array<string, array<string, scalar|array<scalar>>>} $linter_config
 */
function generate_category_files(array $rules_by_category, string $rules_target_dir, array $linter_config): void
{
    writeln('✍️ ', 'Generating documentation file for each category...');
    foreach ($rules_by_category as $category_name => $data) {
        $file_path = $rules_target_dir . '/' . $data['kebab'] . '.md';
        $category_content = create_category_markdown_content($category_name, $data['rules'], $linter_config);
        file_put_contents($file_path, $category_content);
    }
}

/**
 * Generates the main `rules-and-categories.md` overview page.
 *
 * @param array<string, array{kebab: string, rules: list<array{
 *   name: string,
 *   code: string,
 *   description: string,
 *   good_example: string,
 *   bad_example: string,
 *   category: string,
 *   requires: int,
 *   php: array{min: ?string, max: ?string}
 * }>}> $rules_by_category
 *
 * @param array<string, list<array{
 *   name: string,
 *   code: string,
 *   description: string,
 *   good_example: string,
 *   bad_example: string,
 *   category: string,
 *   requires: int,
 *   php: array{min: ?string, max: ?string}
 * }>> $rules_by_integration
 *
 * @mago-expect lint:no-else-clause
 */
function generate_overview_page(string $docs_dir, array $rules_by_category, array $rules_by_integration): void
{
    $overviewPagePath = $docs_dir . '/tools/linter/rules-and-categories.md';
    writeln('✍️ ', 'Generating main overview page: %s', $overviewPagePath);

    $overviewContent = "---\nsidebar_position: 3\ntitle: Rules & categories\n---\n\n# Rules & categories\n\n";
    $overviewContent .= "**Mago**'s linter comes with a wide variety of rules, each designed to catch a specific type of issue.\n\n";
    $overviewContent .= "## Rule categories\n\n";
    foreach ($rules_by_category as $categoryName => $data) {
        $overviewContent .= "- [{$categoryName}](./rules/{$data['kebab']})\n";
    }

    $overviewContent .= "\n## Integration-specific rules\n\n";
    if ([] === $rules_by_integration) {
        $overviewContent .= "There are currently no rules that require a specific integration.\n";
    } else {
        foreach ($rules_by_integration as $integrationName => $integrationRules) {
            $overviewContent .= "\n### {$integrationName}\n\n";
            foreach ($integrationRules as $rule) {
                $categoryKebab = to_kebab_case($rule['category']);
                $overviewContent .= "- [`{$rule['code']}`](./rules/{$categoryKebab}#{$rule['code']})\n";
            }
        }
    }

    file_put_contents($overviewPagePath, $overviewContent);
}

/**
 * Creates the full markdown content for a single category file.
 *
 * @param list<array{
 *   name: string,
 *   code: string,
 *   description: string,
 *   good_example: string,
 *   bad_example: string,
 *   category: string,
 *   requires: int,
 *   php: array{min: ?string, max: ?string}
 * }> $rules
 *
 * @param array{rules: array<string, array<string, scalar|array<scalar>>>} $linter_config
 */
function create_category_markdown_content(string $category_name, array $rules, array $linter_config): string
{
    usort($rules, fn(array $a, array $b): int => $a['code'] <=> $b['code']);

    $content = <<<MD
    ---
    title: {$category_name} rules
    outline: [2, 3]
    ---

    # {$category_name} rules

    This document details the rules available in the `{$category_name}` category.

    | Rule | Code |
    | :--- | :---------- |
    MD;

    foreach ($rules as $rule) {
        $content .= "\n| {$rule['name']} | [`{$rule['code']}`](#{$rule['code']}) |";
    }

    $content .= "\n\n";

    foreach ($rules as $rule) {
        $rule_config = $linter_config['rules'][$rule['code']] ?? ['enabled' => true, 'level' => 'error'];
        $content .= "\n" . generate_rule_docs_section($rule, $rule_config) . "\n\n";
    }

    return $content;
}

/**
 * Creates the markdown section for a single rule.
 *
 * @param array{
 *   name: string,
 *   code: string,
 *   description: string,
 *   good_example: string,
 *   bad_example: string,
 *   category: string,
 *   requires: int,
 *   php: array{min: ?string, max: ?string}
 * } $rule
 * @param array<string, scalar|array<scalar>> $config
 *
 * @mago-expect lint:no-else-clause
 * @mago-expect lint:halstead
 */
function generate_rule_docs_section(array $rule, array $config): string
{
    $good_example = trim($rule['good_example']);
    $bad_example = trim($rule['bad_example']);

    if ('' === $good_example && '' === $bad_example) {
        writeln('⚠️ ', 'Missing examples for rule: %s', $rule['code']);
    }

    $requirements_items = [];
    $requiredIntegrations = [];
    if ($rule['requires'] > 0) {
        foreach (Integration::cases() as $case) {
            if (($rule['requires'] >> $case->value) & 1) {
                $requiredIntegrations[] = '`' . $case->name . '`';
            }
        }
    }

    $phpMin = $rule['php']['min'];
    $phpMax = $rule['php']['max'];
    $phpVersion = '';
    if (null !== $phpMin && null !== $phpMax) {
        $phpVersion = "PHP `{$phpMin}` - `{$phpMax}`";
    } elseif (null !== $phpMin) {
        $phpVersion = "PHP `>= {$phpMin}`";
    } elseif (null !== $phpMax) {
        $phpVersion = "PHP `< {$phpMax}`";
    }

    if ($phpVersion) {
        $requirements_items[] = "- **PHP version:** {$phpVersion}";
    }
    if ([] !== $requiredIntegrations) {
        $plural = count($requiredIntegrations) > 1 ? 's' : '';
        $requirements_items[] = "- **Integration{$plural}:** " . implode(', ', $requiredIntegrations);
    }

    $requirements_section = '';
    if ([] !== $requirements_items) {
        $requirements_section = "### Requirements\n\n" . implode("\n", $requirements_items) . "\n";
    }

    $config_table = "### Configuration\n\n";
    $config_table .= "| Option | Type | Default |\n";
    $config_table .= "| :--- | :--- | :--- |\n";
    foreach ($config as $key => $value) {
        $type = gettype($value);
        if ($key === 'level' && is_string($value)) {
            $value = strtolower($value);
        }
        $default_value = json_encode($value);
        $config_table .= "| `{$key}` | `{$type}` | `{$default_value}` |\n";
    }

    $examples_section = '';
    if ('' !== $good_example || '' !== $bad_example) {
        $examples_section .= "### Examples\n\n";
        if ('' !== $good_example) {
            $examples_section .= "#### Correct code\n\n```php\n{$good_example}\n```\n\n";
        }
        if ('' !== $bad_example) {
            $examples_section .= "#### Incorrect code\n\n```php\n{$bad_example}\n```\n\n";
        }
    }

    return sprintf(
        "## <a id=\"%s\"></a>`%s`\n\n%s\n\n%s\n%s\n%s",
        $rule['code'],
        $rule['code'],
        $rule['description'],
        $requirements_section,
        $config_table,
        rtrim($examples_section),
    );
}

/**
 * Converts a string from PascalCase or Title Case to kebab-case.
 */
function to_kebab_case(string $str): string
{
    $kebab = preg_replace('/([a-z0-9]|(?=[A-Z]))([A-Z])/', '$1-$2', $str);
    if (null === $kebab) {
        throw new RuntimeException("Failed to convert string to kebab-case: {$str}");
    }

    return trim(str_replace(' ', '-', strtolower($kebab)), '-');
}

/**
 * Recursively deletes a directory and all its contents.
 *
 * @mago-expect lint:no-boolean-literal-comparison
 */
function delete_directory(string $dir): void
{
    if (!is_dir($dir)) {
        return;
    }

    $files = scandir($dir);
    if ($files === false) {
        throw new RuntimeException("Failed to read directory: {$dir}");
    }

    $files = array_diff($files, ['.', '..']);
    foreach ($files as $file) {
        is_dir("{$dir}/{$file}") ? delete_directory("{$dir}/{$file}") : unlink("{$dir}/{$file}");
    }

    rmdir($dir);
}

function writeln(string $prefix, string $message, string|int|float ...$args): void
{
    fwrite(STDERR, sprintf("{$prefix} {$message}\n", ...$args));
}
