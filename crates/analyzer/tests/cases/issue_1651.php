<?php

declare(strict_types=1);

final class ImageAnalysisSignature
{
    /** @var int[] $hist8_red */
    public array $hist8_red;
    /** @var int[] $hist8_v */
    public array $hist8_v;

    public function __construct()
    {
        $a = [0, 0, 0, 0, 0, 0, 0, 0];
        $this->hist8_red = $a;
        $this->hist8_v = $a;
    }
}

function image_analysis_color_calc_id(ImageAnalysisSignature $info): void
{
    $ch = ['red', 'v'];
    foreach ($ch as $c) {
        $m = 0;
        $mp = 0;
        for ($i = 0; $i < 8; ++$i) {
            $k = "hist8_{$c}";
            if ($info->$k[$i] > $m) {
                $m = $info->$k[$i];
                $mp = $i;
            }
        }
    }
}
