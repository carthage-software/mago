<?php

declare(strict_types=1);

class ImageWidgetConfig {}

class TextWidgetConfig {}

class MarkupWidgetConfig {}

/**
 * @type NonInteractiveWidgetConfig = ImageWidgetConfig|TextWidgetConfig|MarkupWidgetConfig
 */
class StepWidgetConfig {}

// @mago-expect analysis:missing-constructor
class SegmentsConfig
{
    /**
     * @var list<!StepWidgetConfig::NonInteractiveWidgetConfig>
     */
    public array $header;
}

function test(): void
{
    /** @var list<!StepWidgetConfig::NonInteractiveWidgetConfig> $header */
    $header = [];

    $l = [1, 2, 3];
    foreach ($l as $step) {
        /** @var false|!StepWidgetConfig::NonInteractiveWidgetConfig */
        $result = test2();
        if ($result === false) {
            continue;
        }

        $header[] = $result;
    }

    $segments = new SegmentsConfig();
    $segments->header = $header;
}

function test2(): mixed
{
    return false;
}
