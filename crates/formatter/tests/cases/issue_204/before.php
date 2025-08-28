<?php

function a() {
    $promises = (/**
     * @return \Generator
     * @return \Generator
     */
    static function () use ($paginator): \Generator {
        yield $paginator->getCurrentPageResultsAsync();

        while ($paginator->hasNextPage()) {
            $paginator->nextPage();

            yield $paginator->getCurrentPageResultsAsync();
        }
    }
    )();
}

function b()
{
    Psl\invariant(null === Iter\search(Vec\concat(...Vec\map($this->section->getRows(), static fn(ProductRow $row): array => $row->getFields())), static fn(ProductField $existing): bool => $existing->getName() === $field->getName()), 'Field names need to be unique per section.');
}

function c()
{
    [
        'claim_coverage_descriptions' => $claimCoverageDescriptions = Vec\map($filter->claimCoverageDescriptions, static fn(ClaimCoverageDescription $claimCoverageDescription): string => $claimCoverageDescription->value),
    ];
}

function d()
{
    return Iter\search($infos, static fn(ProductCommercialPropertyInfo $existing): bool => $existing->getSubLocation()->getLocation()->getUuid()->equals($location->getUuid()) && $existing->getSubLocation()->getNumber() === $subLocationNumber)
        ?? Iter\search($infos, static fn(ProductCommercialPropertyInfo $existing): bool => $existing->getSubLocation()->getLocation()->getNumber() === $location->getNumber() && $existing->getSubLocation()->getNumber() === $subLocationNumber)
        ?? null;
}

function e()
{
    foreach ($target->getQuoteTemplateDefaults()->getFormSpecifications() as $formSpecification) {
        if (null === Iter\search($selectedFormSpecifications, static fn(FormSpecificationDTO $possible): bool => $formSpecification->getDocumentTemplateConfiguration() === $possible->documentTemplateConfiguration)) {
            $target->getQuoteTemplateDefaults()->removeFormSpecification($formSpecification);
        }
    }
}

function getProductCount($division): int{
    return 1;
}

function getFormsStep(Division $division): int
{
    return  3 + // foo
        +\count([1,2,3])
        + 1 // bar
        + 1 // baz
        + getProductCount($division)
        + 1; // bay
}

function f()
{
    if(true) {
        $a = !$subject->dto->inBusinessGreaterThanThreeYears ? Eligibility::create(
            EligibilityStatus::REFER,
            ['Builders in the construction business for less than 3 years require an underwriter to review. A resume showing related construction experience will be required.'],
        ) : $currentEligibility;
    }
}

function g()
{
    $coverageCodes = isset($options['coverage'])
        ? [$options['coverage']]
        /** @phpstan-ignore offsetAccess.notFound */
        : Type\vec(Type\instance_of(CoverageCode::class))->assert($options['coverage_codes']);
}

function h()
{
    if(true){
        Vec\sort_by(
            Vec\concat(
                ($this->dataPointContextResolver)(
                    [$context],
                    $dataPointReplacements,
                    static fn(TemplateReplacementInterface $templateReplacement) => Type\instance_of(DataPointDataSource::class)->assert($templateReplacement->getDataSource())->value,
                ),
                $this->getProductReplacementsFor($context),
            ),
            static fn(TemplateReplacementInterface $replacement): string => $replacement->getType()->getReadable(),
        );
    }

}