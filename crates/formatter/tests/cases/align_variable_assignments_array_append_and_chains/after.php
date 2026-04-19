<?php

$value      = Identifier::fromString($rawId)->toBinary();
$valueIds[] = $value;

$first   = $second = new DateTimeImmutable();
$payload = $args['input'];

$start = $finish = null;
$rows  = $rowTypes = [];
$types = [];

$primaryItems = $secondaryItems = $archivedItems = $overflowItems = $allItems = [];
$hasPrimary   = $hasSecondary = $hasArchived = $hasOverflow = false;

$totalCount    = $data['total'];
$matchedCount  = $data['matched'];
$ratio         = $totalCount > 0 ? $matchedCount / $totalCount : 0.0;
$reportItems[] = new DailyRatioItem(
    $data['day'],
    $ratio,
    $totalCount,
    $matchedCount,
);
$periodCount = count($reportItems);
$averageRate = $periodCount > 0
    ? $sumRatio / $periodCount
    : 0.0;
$totalMatched = array_reduce(
    $reportItems,
    fn ($carry, $item) => $carry + $item->getMatchedCount(),
    0,
);

$averageLeft = null !== $row['averageLeft']
    ? (int) round((float) $row['averageLeft'])
    : null;
$averageRight = null !== $row['averageRight']
    ? (int) round((float) $row['averageRight'])
    : null;
$dayTotal     = (int) $row['total'];
$dayWithValue = (int) $row['withValue'];
$dayDone      = (int) $row['done'];
