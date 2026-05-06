<?php

declare(strict_types=1);

/**
 * @psalm-type Coord = array{x: int, y: int}
 */
final class GeoBox
{
    /**
     * @param Coord $c
     * @return Coord
     */
    public function move(array $c, int $dx, int $dy): array
    {
        return ['x' => $c['x'] + $dx, 'y' => $c['y'] + $dy];
    }
}

$g = new GeoBox();
$out = $g->move(['x' => 1, 'y' => 2], 3, 4);
echo $out['x'];
