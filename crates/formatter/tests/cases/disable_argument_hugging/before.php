<?php

class Example
{
    function run()
    {
        return HttpResponse::create(data: ['job' => AcceptedOrderResource::make($queue->retry(getActor(), $job))], status: 202);
    }

    function notify()
    {
        $user->notify((new OrderShippedMessage($job, $order, $subscription))->locale($locale)->delay(now()->addSeconds(30)));
        $user->notify(new OrderShippedMessage(subscriptionId: $subscription->id, jobId: $job->id, messageSequence: $job->deliveryAttempts->sequence, title: $subscription->title ?? $recipientName, body: $body)->locale($locale)->delay(now()->addSeconds((int) config('notifications.order_delay_seconds'))));
        $user->notify((new OrderShippedMessage(job: $job, order: $order, subscription: $subscription))->locale($locale)->delay(now()->addSeconds(30)));
    }
}

short_call(name: 'value', status: 202);
short_call(inner($a, $b));
short_call(new Example($a, $b));
