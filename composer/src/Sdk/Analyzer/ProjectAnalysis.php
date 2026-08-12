<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * Final merged whole-codebase analysis result.
 *
 * @api
 */
final class ProjectAnalysis
{
    /**
     * @var array<string, FileAnalysis>
     */
    private readonly array $byFile;

    /**
     * @param list<FileAnalysis> $files
     */
    public function __construct(
        public readonly array $files,
        public readonly int $issueCount,
        public readonly ReferenceSummary $references,
    ) {
        $byFile = [];
        foreach ($files as $file) {
            $byFile[$file->file] = $file;
        }

        $this->byFile = $byFile;
    }

    public function getFile(string $file): ?FileAnalysis
    {
        return $this->byFile[$file] ?? null;
    }

    /**
     * @param list<string> $files
     *
     * @return list<FileAnalysis|null>
     */
    public function getMultipleFiles(array $files): array
    {
        $analyses = [];
        foreach ($files as $file) {
            $analyses[] = $this->byFile[$file] ?? null;
        }

        return $analyses;
    }
}
