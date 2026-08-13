<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Fixtures;

use Mago\Sdk\Analyzer\AfterFileAnalysisContext;
use Mago\Sdk\Analyzer\AfterFileAnalysisHook;
use Mago\Sdk\Analyzer\BeforeAnalysisContext;
use Mago\Sdk\Analyzer\BeforeAnalysisHook;
use Mago\Sdk\Analyzer\Definition\ConstantDefinition;
use Mago\Sdk\Analyzer\Plugin;
use Mago\Sdk\Analyzer\PluginDefinition;
use Mago\Sdk\Analyzer\PluginRegistry;
use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\Extension;
use Mago\Sdk\Worker;
use RuntimeException;

use function dirname;
use function file_put_contents;
use function getenv;
use function getmypid;
use function is_string;
use function json_encode;

use const FILE_APPEND;
use const JSON_THROW_ON_ERROR;
use const LOCK_EX;

require_once dirname(__DIR__, 4) . '/vendor/autoload.php';

final class GenerationMutationHook implements BeforeAnalysisHook, AfterFileAnalysisHook
{
    private int $generation = 0;

    public function __construct(
        private readonly string $auditLog,
    ) {}

    public function beforeAnalysis(BeforeAnalysisContext $context): void
    {
        $this->generation++;
        if ($this->generation === 1) {
            $context->codebase->insertConstant(new ConstantDefinition('EPHEMERAL_EXTENSION_VALUE', Type::int()));
        }

        $this->record('before', null);
    }

    public function afterFileAnalysis(AfterFileAnalysisContext $context): void
    {
        $this->record('after-file', $context->analysis->file);
    }

    private function record(string $phase, ?string $file): void
    {
        $record = json_encode(['conditional', $phase, $file, getmypid()], JSON_THROW_ON_ERROR);
        if (file_put_contents($this->auditLog, $record . "\n", FILE_APPEND | LOCK_EX) === false) {
            throw new RuntimeException('Unable to append to the mutation generation audit log.');
        }
    }
}

/** @mago-expect lint:single-class-per-file */
final class GenerationMutationPlugin implements Plugin
{
    public function __construct(
        private readonly string $auditLog,
    ) {}

    public function getDefinition(): PluginDefinition
    {
        return new PluginDefinition('conditional', 'Conditional mutation', 'Changes metadata between generations.');
    }

    public function register(PluginRegistry $registry): void
    {
        $hook = new GenerationMutationHook($this->auditLog);
        $registry->registerBeforeAnalysisHook($hook);
        $registry->registerAfterFileAnalysisHook($hook);
    }
}

$auditLog = getenv('MAGO_MUTATION_GENERATION_AUDIT_LOG');
if (!is_string($auditLog) || $auditLog === '') {
    throw new RuntimeException('MAGO_MUTATION_GENERATION_AUDIT_LOG must name the audit file.');
}

(new Worker(new Extension(
    identifier: 'mago/mutation-generation-proof',
    name: 'Mago analyzer mutation generation proof',
    version: '1.0.0',
    analyzerPlugins: [new GenerationMutationPlugin($auditLog)],
)))->run();
