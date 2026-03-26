<?php

declare(strict_types=1);

namespace Example;

interface SubjectInterface
{
    /**
     * @param ObserverInterface<static> $observer
     */
    public function subscribe(ObserverInterface $observer): void;

    /**
     * @param ObserverInterface<static> $observer
     */
    public function unsubscribe(ObserverInterface $observer): void;

    public function notify(): void;
}

/**
 * @template T of SubjectInterface
 */
interface ObserverInterface
{
    /**
     * @param T $subject
     */
    public function update(SubjectInterface $subject): void;
}

final class Inventory implements SubjectInterface
{
    public function subscribe(ObserverInterface $observer): void
    {
       exit('not important');
    }

    public function unsubscribe(ObserverInterface $observer): void
    {
       exit('not important');
    }

    public function notify(): void
    {
       exit('not important');
    }

    public function restock(int $_quantity): void
    {
       exit('not important');
    }
}

/** @implements ObserverInterface<Inventory> */
final class StockAlert implements ObserverInterface
{
    public function update(SubjectInterface $subject): void
    {
       exit('not important');
    }
}

$inventory = new Inventory();
$alert = new StockAlert();

$inventory->subscribe($alert);
$inventory->restock(150);
$inventory->unsubscribe($alert);
$inventory->restock(50);
