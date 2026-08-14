<?php

declare(strict_types=1);

namespace Mago\Sdk;

use Mago\Sdk\Analyzer\Plugin;
use Mago\Sdk\Exception\InvalidArgumentException;
use Mago\Sdk\Linter\Rule;

use function array_key_exists;
use function strtolower;

/**
 * A Mago extension and the capabilities it contributes.
 *
 * @api
 * @mago-expect lint:cyclomatic-complexity
 * @mago-expect lint:excessive-parameter-list
 */
final class Extension
{
    /**
     * @var non-empty-string
     */
    public readonly string $identifier;

    /**
     * @var non-empty-string
     */
    public readonly string $name;

    /**
     * @var non-empty-string
     */
    public readonly string $version;

    /**
     * @var list<Rule>
     */
    public readonly array $linterRules;

    /** @var list<Plugin> */
    public readonly array $analyzerPlugins;

    public readonly ?WorkerReducer $workerReducer;

    /**
     * @param string $identifier Stable, globally unique extension identifier.
     * @param string $name Human-readable extension name.
     * @param string $version Extension package version.
     * @param list<Rule> $linterRules
     * @param list<Plugin> $analyzerPlugins
     */
    public function __construct(
        string $identifier,
        string $name,
        string $version,
        array $linterRules = [],
        array $analyzerPlugins = [],
        ?WorkerReducer $workerReducer = null,
    ) {
        if ($identifier === '') {
            throw new InvalidArgumentException('An extension identifier cannot be empty.');
        }

        if ($name === '') {
            throw new InvalidArgumentException('An extension name cannot be empty.');
        }

        if ($version === '') {
            throw new InvalidArgumentException('An extension version cannot be empty.');
        }

        $codes = [];
        foreach ($linterRules as $rule) {
            $code = $rule->getDefinition()->code;
            if (array_key_exists($code, $codes)) {
                throw new InvalidArgumentException(
                    "Extension `{$identifier}` registers linter rule `{$code}` more than once.",
                );
            }

            $codes[$code] = true;
        }

        $this->identifier = $identifier;
        $this->name = $name;
        $this->version = $version;
        $this->linterRules = $linterRules;
        $pluginIdentifiers = [];
        foreach ($analyzerPlugins as $plugin) {
            $pluginIdentifier = strtolower($plugin->getDefinition()->identifier);
            if (array_key_exists($pluginIdentifier, $pluginIdentifiers)) {
                throw new InvalidArgumentException(
                    "Extension `{$identifier}` registers analyzer plugin `{$pluginIdentifier}` more than once.",
                );
            }

            $pluginIdentifiers[$pluginIdentifier] = true;
        }

        $this->analyzerPlugins = $analyzerPlugins;
        $this->workerReducer = $workerReducer;
    }
}
