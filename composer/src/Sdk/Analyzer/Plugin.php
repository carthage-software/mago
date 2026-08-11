<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * A custom analyzer plugin that registers semantic providers and hooks.
 *
 * @api
 */
interface Plugin
{
    public function getDefinition(): PluginDefinition;

    public function register(PluginRegistry $registry): void;
}
