<?php

declare(strict_types=1);

const DIRECTIONS = ['IN', 'OUT', 'INOUT'];

/** @param array<string> $paramsDir */
function setParams(array $paramsDir): void {}

function bug(mixed $rawDir, bool $isProcedure): void
{
    if ($isProcedure) {
        $itemParamDir = is_array($rawDir) ? $rawDir : [];
        // @mago-expect analysis:mixed-assignment
        foreach ($itemParamDir as $key => $value) {
            if (in_array($value, DIRECTIONS, true)) {
                continue;
            }

            $itemParamDir[$key] = '';
        }
    }

    setParams($itemParamDir ?? []);
}

function breakDoesNotSanitizeRemainingValues(mixed $rawDir): void
{
    $itemParamDir = is_array($rawDir) ? $rawDir : [];
    // @mago-expect analysis:mixed-assignment
    foreach ($itemParamDir as $key => $value) {
        if (in_array($value, DIRECTIONS, true)) {
            break;
        }

        $itemParamDir[$key] = '';
    }

    // @mago-expect analysis:less-specific-nested-argument-type
    setParams($itemParamDir);
}

function continuePathCanAddMixedValues(mixed $rawDir): void
{
    $itemParamDir = is_array($rawDir) ? $rawDir : [];
    // @mago-expect analysis:mixed-assignment
    foreach ($itemParamDir as $key => $value) {
        if (in_array($value, DIRECTIONS, true)) {
            foreach ([0] as $_) {
                $itemParamDir['extra'] = $rawDir;
            }

            continue;
        }

        $itemParamDir[$key] = '';
    }

    // @mago-expect analysis:less-specific-nested-argument-type
    setParams($itemParamDir);
}
