<?php

declare(strict_types=1);

/** @template T1 */
abstract class Source
{
    /**
     * @return T1
     *
     * @throws Exception
     */
    public function get(): mixed
    {
        throw new Exception();
    }
}

/**
 * @template T2
 *
 * @extends Source<list<T2>>
 */
abstract class ListSource extends Source
{
}

/** @extends ListSource<string> */
final class Names extends ListSource
{
}

/**
 * @template T3
 *
 * @extends ListSource<T3>
 */
final class Ids extends ListSource
{
}

/** @throws Exception */
function lengths(Names $names): int
{
    $total = 0;
    foreach ($names->get() as $name) {
        $total += strlen($name);
    }

    return $total;
}

/**
 * @param Ids<int> $ids
 *
 * @throws Exception
 */
function total(Ids $ids): int
{
    $sum = 0;
    foreach ($ids->get() as $id) {
        $sum += $id;
    }

    return $sum;
}

/**
 * @param ListSource<string> $source
 *
 * @return list<string>
 *
 * @throws Exception
 */
function one_level(ListSource $source): array
{
    return $source->get();
}

/** @template T1 */
interface IBase
{
    /** @return T1 */
    public function value(): mixed;
}

/**
 * @template T2
 *
 * @extends IBase<list<T2>>
 */
interface IMid extends IBase
{
}

/** @implements IMid<string> */
final class Words implements IMid
{
    public function value(): array
    {
        return ['a'];
    }
}

function word_lengths(Words $words): int
{
    $total = 0;
    foreach ($words->value() as $word) {
        $total += strlen($word);
    }

    return $total;
}
