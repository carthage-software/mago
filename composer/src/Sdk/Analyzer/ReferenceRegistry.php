<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\Analyzer\Metadata\MemberIdentifier;
use Mago\Sdk\Exception\InvalidArgumentException;

/**
 * Collects framework-known symbol references.
 *
 * A before-analysis registry contributes project-wide references. An after-file
 * or node-analysis registry contributes references scoped to the analyzed file.
 *
 * @api
 */
final class ReferenceRegistry
{
    /**
     * Stored flat to keep the hot response encoder allocation-light.
     *
     * @var list<string|MemberIdentifier|ReferenceOrigin|ReferenceKind>
     */
    private array $references = [];

    public function add(
        string|MemberIdentifier|ReferenceOrigin $source,
        string|MemberIdentifier $target,
        ReferenceKind $kind = ReferenceKind::Body,
    ): void {
        if ($source === '' || $target === '') {
            throw new InvalidArgumentException('Reference source and target symbols cannot be empty.');
        }

        $this->references[] = $source;
        $this->references[] = $target;
        $this->references[] = $kind;
    }

    public function addPropertyRead(string|MemberIdentifier $source, MemberIdentifier $property): void
    {
        $this->add($source, $property, ReferenceKind::PropertyRead);
    }

    public function addPropertyWrite(string|MemberIdentifier $source, MemberIdentifier $property): void
    {
        $this->add($source, $property, ReferenceKind::PropertyWrite);
    }

    public function addOverriddenMember(MemberIdentifier $source, MemberIdentifier $overridden): void
    {
        $this->add($source, $overridden, ReferenceKind::OverriddenMember);
    }

    public function addFunctionLikeReturn(string|MemberIdentifier $source, string|MemberIdentifier $target): void
    {
        $this->add($source, $target, ReferenceKind::FunctionLikeReturn);
    }

    /**
     * @internal
     *
     * @return list<string|MemberIdentifier|ReferenceOrigin|ReferenceKind>
     */
    public function takeReferences(): array
    {
        $references = $this->references;
        $this->references = [];

        return $references;
    }
}
