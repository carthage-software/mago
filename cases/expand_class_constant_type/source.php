<?php

class WorkerMessageFailedEvent {}

class WorkerMessageHandledEvent {}

class KernelEvents
{
    public const string TERMINATE = 'kernel.terminate';
}

class ConsoleEvents
{
    public const string TERMINATE = 'console.terminate';
}

class EmailEvents
{
    public const string EMAIL_RECEIVED = 'email.received';
}

class ImportEvents
{
    public const string IMPORT_ITEM_PROCESSED = 'import.item_processed';
}

class SomeServiceListener
{
    private const array EVENTS = [
        KernelEvents::TERMINATE,
        ConsoleEvents::TERMINATE,
        EmailEvents::EMAIL_RECEIVED,
        WorkerMessageFailedEvent::class,
        WorkerMessageHandledEvent::class,
        ImportEvents::IMPORT_ITEM_PROCESSED,
    ];

    /**
     * @return array<string, 'handle'>
     */
    public static function getSubscribedEvents(): array
    {
        $events = [];
        foreach (self::EVENTS as $event) {
            $events[$event] = 'handle';
        }

        return $events;
    }
}
