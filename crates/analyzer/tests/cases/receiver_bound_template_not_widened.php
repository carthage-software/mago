<?php

declare(strict_types=1);

/** @template-covariant T */
interface ReceiverBox
{
    /** @return T */
    public function get(): mixed;
}

/** @template T */
final class ReceiverSink
{
    /** @param ReceiverBox<T> $box */
    public function putBox(ReceiverBox $box): void {}

    /** @param list<T> $list */
    public function putList(array $list): void {}
}

/**
 * @param ReceiverSink<int> $sink
 * @param ReceiverBox<string> $box
 * @param list<string> $list
 */
function receiver_bound_fill(ReceiverSink $sink, ReceiverBox $box, array $list): void
{
    // @mago-expect analysis:possibly-invalid-argument
    $sink->putBox($box);
    // @mago-expect analysis:invalid-argument
    $sink->putList($list);
}
