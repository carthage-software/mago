<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Fixtures;

use Mago\Sdk\Analyzer\CodebaseScanContext;
use Mago\Sdk\Analyzer\CodebaseScanHook;
use Mago\Sdk\Analyzer\Plugin;
use Mago\Sdk\Analyzer\PluginDefinition;
use Mago\Sdk\Analyzer\PluginRegistry;
use Mago\Sdk\Analyzer\SourceFileTarget;
use Mago\Sdk\Extension;
use Mago\Sdk\Syntax\NodeKind;
use Mago\Sdk\Worker;
use RuntimeException;

use function array_keys;
use function dirname;
use function file_put_contents;
use function getenv;
use function getmypid;
use function is_string;
use function json_encode;
use function sort;

use const FILE_APPEND;
use const JSON_THROW_ON_ERROR;
use const LOCK_EX;

require_once dirname(__DIR__, 4) . '/vendor/autoload.php';

/**
 * @mago-expect lint:cyclomatic-complexity
 */
final class CodebaseScanProofHook implements CodebaseScanHook
{
    /** @var array<string, list<string>> */
    private array $values = [];

    public function __construct(
        private readonly string $auditLog,
    ) {}

    public function getTargets(): array
    {
        return [new SourceFileTarget('database/migrations/**/*.php')];
    }

    public function scan(CodebaseScanContext $context): void
    {
        if ($context->firstBatch) {
            $this->values = [];
        }

        foreach ($context->files as $source) {
            $values = [];
            foreach ($source->getNodes(NodeKind::LiteralString) as $literal) {
                $value = $source->getLiteralString($literal);
                if ($value !== null) {
                    $values[] = $value;
                }
            }
            $this->values[$source->path] = $values;
        }

        if (!$context->lastBatch) {
            return;
        }

        $paths = array_keys($this->values);
        sort($paths);
        if ($paths !== ['database/migrations/001.php', 'database/migrations/nested/002.php']) {
            throw new RuntimeException('The codebase-scan hook received an unexpected source selection.');
        }

        $record = json_encode([getmypid(), $this->values], JSON_THROW_ON_ERROR);
        if (file_put_contents($this->auditLog, $record . "\n", FILE_APPEND | LOCK_EX) === false) {
            throw new RuntimeException('Unable to append to the codebase-scan audit log.');
        }
    }
}

/** @mago-expect lint:single-class-per-file */
final class CodebaseScanProofPlugin implements Plugin
{
    public function __construct(
        private readonly string $auditLog,
    ) {}

    public function getDefinition(): PluginDefinition
    {
        return new PluginDefinition('codebase-scan-proof', 'Codebase scan proof', 'Exercises filtered source scans.');
    }

    public function register(PluginRegistry $registry): void
    {
        $registry->registerCodebaseScanHook(new CodebaseScanProofHook($this->auditLog));
    }
}

/** @mago-expect lint:single-class-per-file */
final class DisabledCodebaseScanHook implements CodebaseScanHook
{
    public function getTargets(): array
    {
        return [new SourceFileTarget('database/migrations/**/*.php')];
    }

    public function scan(CodebaseScanContext $context): void
    {
        throw new RuntimeException('A disabled plugin received a codebase-scan batch.');
    }
}

/** @mago-expect lint:single-class-per-file */
final class DisabledCodebaseScanPlugin implements Plugin
{
    public function getDefinition(): PluginDefinition
    {
        return new PluginDefinition(
            'disabled-codebase-scan-proof',
            'Disabled codebase scan proof',
            'Must never receive selected source.',
            defaultEnabled: false,
        );
    }

    public function register(PluginRegistry $registry): void
    {
        $registry->registerCodebaseScanHook(new DisabledCodebaseScanHook());
    }
}

$auditLog = getenv('MAGO_SCAN_AUDIT_LOG');
if (!is_string($auditLog) || $auditLog === '') {
    throw new RuntimeException('MAGO_SCAN_AUDIT_LOG must name the codebase-scan audit file.');
}

(new Worker(new Extension(
    identifier: 'mago/codebase-scan-proof',
    name: 'Mago codebase-scan proof',
    version: '1.0.0',
    analyzerPlugins: [new CodebaseScanProofPlugin($auditLog), new DisabledCodebaseScanPlugin()],
)))->run();
