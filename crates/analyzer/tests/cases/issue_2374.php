<?php

declare(strict_types=1);

namespace Issue2374;

use function array_pop;
use function array_push;
use function array_shift;
use function array_splice;
use function array_unshift;
use function sort;

final class Repro
{
    /** @var list<array{flags: array<string, true>}> */
    private array $viaArrayPush = [];

    public function arrayStringTrueViaArrayPush(): void
    {
        array_push($this->viaArrayPush, ['flags' => []]);
    }

    /** @var list<array{flags: array<string, true>}> */
    private array $viaPop = [];

    public function arrayStringTrueViaPop(): void
    {
        $this->viaPop[] = ['flags' => []];
        array_pop($this->viaPop);
    }

    /** @var list<array{enabled: true}> */
    private array $shapeTrueViaPop = [];

    public function shapeTrueLiteralViaPop(): void
    {
        $this->shapeTrueViaPop[] = ['enabled' => true];
        array_pop($this->shapeTrueViaPop);
    }

    /** @var list<true> */
    private array $listTrueViaPop = [];

    public function listTrueViaPop(): void
    {
        $this->listTrueViaPop[] = true;
        array_pop($this->listTrueViaPop);
    }

    /** @var list<array{flags: array<string, true>}> */
    private array $viaShift = [];

    public function arrayStringTrueViaShiftControl(): void
    {
        $this->viaShift = [['flags' => []]];
        array_shift($this->viaShift);
    }

    public function arrayStringTrueViaShift(): void
    {
        $this->viaShift[] = ['flags' => []];
        array_shift($this->viaShift);
    }

    /** @var list<array{flags: array<string, true>}> */
    private array $viaSpliceRemove = [];

    public function arrayStringTrueViaSpliceRemoveControl(): void
    {
        $this->viaSpliceRemove = [['flags' => []]];
        array_splice($this->viaSpliceRemove, 0, 1);
    }

    public function arrayStringTrueViaSpliceRemove(): void
    {
        $this->viaSpliceRemove[] = ['flags' => []];
        array_splice($this->viaSpliceRemove, 0, 1);
    }

    /** @var list<array{flags: array<string, true>}> */
    private array $viaSort = [];

    public function arrayStringTrueViaSortControl(): void
    {
        $this->viaSort = [['flags' => []]];
        sort($this->viaSort);
    }

    public function arrayStringTrueViaSort(): void
    {
        $this->viaSort[] = ['flags' => []];
        sort($this->viaSort);
    }

    /** @var list<array{flags: array<string, true>}> */
    private array $viaUnshift = [];

    public function arrayStringTrueViaUnshift(): void
    {
        $this->viaUnshift = [];
        array_unshift($this->viaUnshift, ['flags' => []]);
    }

    /** @var list<array{flags: array<string, true>}> */
    private array $viaSpliceReplace = [];

    public function arrayStringTrueViaSpliceReplace(): void
    {
        $this->viaSpliceReplace = [['flags' => []]];
        array_splice($this->viaSpliceReplace, 0, 1, []);
    }

    /** @var list<array{enabled: false}> */
    private array $shapeFalseViaPop = [];

    public function shapeFalseLiteralViaPop(): void
    {
        $this->shapeFalseViaPop[] = ['enabled' => false];
        array_pop($this->shapeFalseViaPop);
    }
}
